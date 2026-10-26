use std::io::{BufRead, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::{read_screen_answer, Bind, BindDay, PlashkaEvent, Ring, Screen};

const ASK_SCREEN: &str = include_str!("ask_screen.py");

/// The `otkati-ring` process. It is the ring: it remembers the screen answer,
/// holds the encoded minutes, and a bind here saves what it holds.
pub fn run_ring_process() {
    let folder = std::env::var("OTKATI_OTKAT_FOLDER")
        .map(PathBuf::from)
        .unwrap_or_else(|_| Ring::default_otkat_folder());
    let home = std::env::var("OTKATI_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| crate::home_dir());
    let ready = std::env::var_os("OTKATI_READY").is_some();
    let day = std::env::var("OTKATI_BIND_DAY").ok();
    let clock = move || match day.as_deref() {
        Some(text) => parse_bind_day(text),
        None => crate::system_bind_day(),
    };
    let mut ring = Ring::with_clock(folder, clock);
    if !ready {
        if let Some(screen) = read_screen_answer(&home) {
            ring.remember_screen(screen);
        } else {
            ring.begin_waiting();
            if std::env::var_os("OTKATI_ASK_SCREEN").is_some() {
                if let Some(screen) = system_screen_question() {
                    ring.persist_at(home.clone());
                    ring.remember_screen(screen);
                }
            }
        }
    }
    if std::env::var_os("OTKATI_HOME").is_some() {
        ring.persist_at(home.clone());
    }
    let capture = if std::env::var_os("OTKATI_CAPTURE").is_some() {
        let screen = ring.monitor().map(|screen| screen.name().to_string());
        screen.and_then(|screen| start_live_capture(&screen, &home, &mut ring))
    } else {
        None
    };
    if std::env::var_os("OTKATI_SERVE").is_some() {
        if let Ok(path) = std::env::var("OTKATI_SOCKET") {
            serve_socket(ring, Path::new(&path), capture);
        } else {
            serve(ring, capture);
        }
    } else {
        // The answer and any minutes already loaded stay in this process.
        std::thread::park();
    }
}

/// The system question. `OTKATI_SCREEN_ASK`, when set, is a program that prints
/// the chosen screen name. Otherwise the ScreenCast portal opens its dialog.
fn system_screen_question() -> Option<Screen> {
    let asked = match std::env::var("OTKATI_SCREEN_ASK") {
        Ok(program) => Command::new(program)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output(),
        Err(_) => {
            let mut child = Command::new("python3")
                .arg("-")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .ok()?;
            let mut stdin = child.stdin.take()?;
            stdin.write_all(ASK_SCREEN.as_bytes()).ok()?;
            drop(stdin);
            child.wait_with_output()
        }
    };
    let Ok(asked) = asked else {
        return None;
    };
    if !asked.status.success() {
        return None;
    }
    let name = String::from_utf8(asked.stdout).ok()?;
    let name = name.trim();
    if name.is_empty() {
        None
    } else {
        Some(Screen::new(name))
    }
}

/// A served ring with `OTKATI_SOCKET` set. Systemd leaves stdin closed, so the
/// ring listens here instead of exiting on that end of file. One line in, one
/// line out, the same words stdin serve already speaks.
struct LiveCapture {
    child: Child,
    ipc: PathBuf,
    started: Instant,
    replay: Arc<Mutex<Option<PathBuf>>>,
}

impl Drop for LiveCapture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn start_live_capture(screen: &str, home: &Path, ring: &mut Ring) -> Option<LiveCapture> {
    let replay_dir = home.join(".local/state/otkati/replay");
    std::fs::create_dir_all(&replay_dir).ok()?;
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let ipc = PathBuf::from(runtime).join("otkati/replay.sock");
    if let Some(dir) = ipc.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::remove_file(&ipc);
    let child = Command::new("gpu-screen-recorder")
        .arg("-w")
        .arg(screen)
        .arg("-f")
        .arg("60")
        .arg("-c")
        .arg("mp4")
        .arg("-r")
        .arg("600")
        .arg("-replay-storage")
        .arg("ram")
        .arg("-cursor")
        .arg("yes")
        .arg("-k")
        .arg("h264")
        .arg("-encoder")
        .arg("gpu")
        .arg("-a")
        .arg("default_output|default_input")
        .arg("-o")
        .arg(&replay_dir)
        .arg("-ipc")
        .arg(&ipc)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .ok()?;
    let replay = Arc::new(Mutex::new(None));
    let slot = Arc::clone(&replay);
    ring.write_otkat_with(move |path, _duration| copy_replay(&slot, path));
    Some(LiveCapture {
        child,
        ipc,
        started: Instant::now(),
        replay,
    })
}

fn copy_replay(replay: &Mutex<Option<PathBuf>>, path: &Path) -> std::io::Result<()> {
    let source = replay
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| std::io::Error::other("the replay was not saved"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(&source, path)?;
    let _ = std::fs::remove_file(source);
    Ok(())
}

impl LiveCapture {
    /// Save what the recorder is holding and hand that time to the ring.
    /// The file itself is copied when the saved otkat is committed.
    fn pour_into(&mut self, ring: &mut Ring) {
        if self.child.try_wait().ok().flatten().is_some() {
            return;
        }
        let seconds = self.started.elapsed().as_secs().min(10 * 60);
        if seconds == 0 {
            return;
        }
        let Some(path) = save_replay(&self.ipc, seconds) else {
            return;
        };
        *self.replay.lock().unwrap() = Some(path);
        ring.hold(Duration::from_secs(seconds));
    }
}

fn save_replay(ipc: &Path, seconds: u64) -> Option<PathBuf> {
    let mut stream = UnixStream::connect(ipc).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .ok()?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .ok()?;
    writeln!(
        stream,
        r#"{{"id":1,"name":"save-replay","data":{{"seconds":{seconds}}}}}"#
    )
    .ok()?;
    stream.flush().ok()?;
    let mut reply = String::new();
    let mut byte = [0u8; 1];
    loop {
        let read = stream.read(&mut byte).ok()?;
        if read == 0 || byte[0] == b'\n' {
            break;
        }
        reply.push(byte[0] as char);
    }
    if !reply.contains("\"result\":\"ok\"") {
        eprintln!("otkati-ring: replay was not saved: {reply}");
        return None;
    }
    let marker = "\"data\":\"";
    let start = reply.find(marker)? + marker.len();
    let end = start + reply[start..].find('"')?;
    let path = PathBuf::from(&reply[start..end]);
    path.is_file().then_some(path)
}

fn serve_socket(mut ring: Ring, path: &Path, mut capture: Option<LiveCapture>) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::remove_file(path);
    let listener = match UnixListener::bind(path) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("otkati-ring: {err}");
            std::thread::park();
            return;
        }
    };
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    let mut pending: Vec<PlashkaEvent> = Vec::new();
    for conn in listener.incoming() {
        let Ok(stream) = conn else {
            continue;
        };
        let _ = answer_one(&mut ring, &mut pending, capture.as_mut(), stream);
    }
}

fn answer_one(
    ring: &mut Ring,
    pending: &mut Vec<PlashkaEvent>,
    capture: Option<&mut LiveCapture>,
    mut stream: UnixStream,
) -> std::io::Result<()> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        let read = stream.read(&mut byte)?;
        if read == 0 {
            return Ok(());
        }
        if byte[0] == b'\n' {
            break;
        }
        if buf.len() < 1024 {
            buf.push(byte[0]);
        }
    }
    let line = String::from_utf8_lossy(&buf);
    let reply = dispatch(ring, pending, capture, line.trim_end());
    writeln!(stream, "{reply}")?;
    stream.flush()?;
    Ok(())
}

fn serve(mut ring: Ring, mut capture: Option<LiveCapture>) {
    let stdin = std::io::stdin();
    let mut reader = stdin.lock();
    let mut line = String::new();
    let mut pending: Vec<PlashkaEvent> = Vec::new();
    loop {
        line.clear();
        let read = reader.read_line(&mut line).unwrap_or(0);
        if read == 0 {
            break;
        }
        let reply = dispatch(&mut ring, &mut pending, capture.as_mut(), line.trim_end());
        println!("{reply}");
        let _ = std::io::stdout().flush();
    }
}

fn dispatch(
    ring: &mut Ring,
    pending: &mut Vec<PlashkaEvent>,
    capture: Option<&mut LiveCapture>,
    line: &str,
) -> String {
    let (command, arg) = line.split_once('\t').unwrap_or((line, ""));
    if command == "bind" {
        if let Some(capture) = capture {
            capture.pour_into(ring);
        }
    }
    match command {
        "query" => format!(
            "{}\t{}\t{}",
            if ring.asks_which_screen() { "1" } else { "0" },
            ring.monitor().map(Screen::name).unwrap_or("-"),
            ring.held.as_secs()
        ),
        "hold" => {
            let secs = arg.parse::<u64>().unwrap_or(0);
            ring.hold(Duration::from_secs(secs));
            "ok".to_string()
        }
        "remember" => {
            if !arg.is_empty() {
                ring.remember_screen(Screen::new(arg));
            }
            "ok".to_string()
        }
        "bind" => bind_line(ring, pending, arg),
        "commit" => commit_pending(ring, pending),
        _ => "none".to_string(),
    }
}

fn bind_line(ring: &mut Ring, pending: &mut Vec<PlashkaEvent>, chord: &str) -> String {
    match ring.bind(Bind::new(chord)) {
        None => "none".to_string(),
        Some(PlashkaEvent::NothingToSave) => "nothing".to_string(),
        Some(PlashkaEvent::NoAccess) => "no-access".to_string(),
        Some(PlashkaEvent::FolderNotCreated) => "folder".to_string(),
        Some(PlashkaEvent::NoSpace) => "no-space".to_string(),
        Some(PlashkaEvent::ClipDidNotSave) => "clip-did-not-save".to_string(),
        Some(PlashkaEvent::RingOff) => "ring-off".to_string(),
        Some(PlashkaEvent::Saved(saved)) => {
            let line = format!("saved\t{}\t{}", saved.name(), saved.duration().as_secs());
            pending.push(PlashkaEvent::Saved(saved));
            line
        }
    }
}

fn commit_pending(ring: &mut Ring, pending: &mut Vec<PlashkaEvent>) -> String {
    if pending.is_empty() {
        return "ok".to_string();
    }
    drop(pending.remove(0));
    match ring.later_plashka() {
        Some(PlashkaEvent::ClipDidNotSave) => "clip-did-not-save".to_string(),
        _ => "ok".to_string(),
    }
}

fn parse_bind_day(text: &str) -> BindDay {
    let mut parts = text.split('-');
    let year = parts.next().unwrap_or("1970").parse().unwrap_or(1970);
    let month = parts.next().unwrap_or("1").parse().unwrap_or(1);
    let day = parts.next().unwrap_or("1").parse().unwrap_or(1);
    BindDay::new(year, month, day)
}
