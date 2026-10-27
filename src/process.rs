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
    let plan = read_plan(&home);
    let capture = if std::env::var_os("OTKATI_CAPTURE").is_some() {
        let screen = ring.monitor().map(|screen| screen.name().to_string());
        screen.and_then(|screen| start_live_capture(&screen, &home, &mut ring, &plan))
    } else {
        None
    };
    if std::env::var_os("OTKATI_SERVE").is_some() {
        if let Ok(path) = std::env::var("OTKATI_SOCKET") {
            serve_socket(ring, Path::new(&path), capture, &home);
        } else {
            serve(ring, capture, &home);
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
#[derive(Clone, PartialEq, Eq)]
struct RecordPlan {
    codec: String,
    bitrate_mbit: u32,
    length_secs: u32,
    share: u32,
    storage: String,
}

impl RecordPlan {
    fn standard() -> Self {
        Self {
            codec: "h264".to_string(),
            bitrate_mbit: 25,
            length_secs: 600,
            share: 100,
            storage: "ram".to_string(),
        }
    }
}

struct LiveCapture {
    child: Child,
    ipc: PathBuf,
    started: Instant,
    replay: Arc<Mutex<Option<PathBuf>>>,
    screen: String,
    home: PathBuf,
    plan: RecordPlan,
}

impl Drop for LiveCapture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn encoder_path(home: &Path) -> PathBuf {
    home.join(".local/state/otkati/encoder")
}

fn knobs_path(home: &Path) -> PathBuf {
    home.join(".local/state/otkati/knobs")
}

fn recorder_codec(choice: &str) -> &'static str {
    match choice.trim() {
        "hevc" | "HEVC" => "hevc",
        "av1" | "AV1" => "av1",
        _ => "h264",
    }
}

fn bitrate_mbit(text: &str) -> Option<u32> {
    let n: u32 = text.trim().parse().ok()?;
    (8..=80).contains(&n).then_some(n)
}

fn length_secs(text: &str) -> Option<u32> {
    let n: u32 = text.trim().parse().ok()?;
    (15..=1200).contains(&n).then_some(n)
}

fn share_percent(text: &str) -> Option<u32> {
    let n: u32 = text.trim().parse().ok()?;
    ((50..=100).contains(&n) && n % 10 == 0).then_some(n)
}

fn storage_name(text: &str) -> Option<&'static str> {
    match text.trim() {
        "ram" | "memory" | "0" => Some("ram"),
        "disk" | "1" => Some("disk"),
        _ => None,
    }
}

fn read_plan(home: &Path) -> RecordPlan {
    let mut plan = RecordPlan::standard();
    let text = std::fs::read_to_string(knobs_path(home)).unwrap_or_default();
    if text.is_empty() {
        if let Ok(old) = std::fs::read_to_string(encoder_path(home)) {
            plan.codec = recorder_codec(old.trim()).to_string();
        }
        return plan;
    }
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "codec" => plan.codec = recorder_codec(value).to_string(),
            "bitrate" => {
                if let Some(n) = bitrate_mbit(value) {
                    plan.bitrate_mbit = n;
                }
            }
            "length" => {
                if let Some(n) = length_secs(value) {
                    plan.length_secs = n;
                }
            }
            "share" => {
                if let Some(n) = share_percent(value) {
                    plan.share = n;
                }
            }
            "place" => {
                if let Some(n) = storage_name(value) {
                    plan.storage = n.to_string();
                }
            }
            _ => {}
        }
    }
    plan
}

fn write_plan(home: &Path, plan: &RecordPlan) {
    let path = knobs_path(home);
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let text = format!(
        "codec={}\nbitrate={}\nlength={}\nshare={}\nplace={}\n",
        plan.codec, plan.bitrate_mbit, plan.length_secs, plan.share, plan.storage
    );
    let _ = std::fs::write(path, text);
}

fn even_side(full: u32, share: u32) -> u32 {
    let scaled = full.saturating_mul(share) / 100;
    let even = scaled & !1;
    if even < 2 {
        2
    } else {
        even
    }
}

fn number_after(text: &str, key: &str) -> Option<u32> {
    let rest = text.get(text.find(key)? + key.len()..)?;
    let digits: String = rest
        .trim_start()
        .trim_start_matches(':')
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

fn frame_in_monitors_json(json: &str, name: &str) -> Option<(u32, u32)> {
    let needle = format!("\"name\": \"{name}\"");
    let start = json.find(&needle)?;
    let rest = &json[start..];
    let end = rest[needle.len()..]
        .find("\"name\":")
        .map(|index| index + needle.len())
        .unwrap_or(rest.len());
    let window = &rest[..end];
    Some((number_after(window, "\"width\":")?, number_after(window, "\"height\":")?))
}

fn monitor_frame(name: &str) -> Option<(u32, u32)> {
    let output = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    frame_in_monitors_json(&text, name)
}

fn recorder_args(
    screen: &str,
    plan: &RecordPlan,
    frame: Option<(u32, u32)>,
    replay_dir: &Path,
    ipc: &Path,
) -> Vec<String> {
    let mut args = vec![
        "-w".into(),
        screen.into(),
        "-f".into(),
        "60".into(),
        "-c".into(),
        "mp4".into(),
        "-r".into(),
        plan.length_secs.to_string(),
        "-replay-storage".into(),
        plan.storage.clone(),
        "-cursor".into(),
        "yes".into(),
        "-k".into(),
        plan.codec.clone(),
        "-encoder".into(),
        "gpu".into(),
        "-bm".into(),
        "cbr".into(),
        "-q".into(),
        (plan.bitrate_mbit * 1000).to_string(),
        "-a".into(),
        "default_output|default_input".into(),
    ];
    if plan.share < 100 {
        if let Some((width, height)) = frame {
            args.push("-s".into());
            args.push(format!(
                "{}x{}",
                even_side(width, plan.share),
                even_side(height, plan.share)
            ));
        }
    }
    args.push("-o".into());
    args.push(replay_dir.display().to_string());
    args.push("-ipc".into());
    args.push(ipc.display().to_string());
    args
}

fn start_live_capture(
    screen: &str,
    home: &Path,
    ring: &mut Ring,
    plan: &RecordPlan,
) -> Option<LiveCapture> {
    let replay_dir = home.join(".local/state/otkati/replay");
    std::fs::create_dir_all(&replay_dir).ok()?;
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let ipc = PathBuf::from(runtime).join("otkati/replay.sock");
    if let Some(dir) = ipc.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::remove_file(&ipc);
    let frame = monitor_frame(screen);
    let child = Command::new("gpu-screen-recorder")
        .args(recorder_args(screen, plan, frame, &replay_dir, &ipc))
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
        screen: screen.to_string(),
        home: home.to_path_buf(),
        plan: plan.clone(),
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
    /// Restart the recorder when a knob changed. The minutes already held are dropped.
    fn apply(&mut self, ring: &mut Ring, plan: RecordPlan) {
        if self.plan == plan {
            return;
        }
        let screen = self.screen.clone();
        let home = self.home.clone();
        let _ = self.child.kill();
        let _ = self.child.wait();
        match start_live_capture(&screen, &home, ring, &plan) {
            Some(next) => *self = next,
            None => eprintln!("otkati-ring: could not restart the recorder"),
        }
    }

    /// Save what the recorder is holding and hand that time to the ring.
    /// The file itself is copied when the saved otkat is committed.
    fn pour_into(&mut self, ring: &mut Ring) {
        if self.child.try_wait().ok().flatten().is_some() {
            return;
        }
        let seconds = self
            .started
            .elapsed()
            .as_secs()
            .min(u64::from(self.plan.length_secs));
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

fn serve_socket(mut ring: Ring, path: &Path, mut capture: Option<LiveCapture>, home: &Path) {
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
        let _ = answer_one(&mut ring, &mut pending, capture.as_mut(), home, stream);
    }
}

fn answer_one(
    ring: &mut Ring,
    pending: &mut Vec<PlashkaEvent>,
    capture: Option<&mut LiveCapture>,
    home: &Path,
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
    let reply = dispatch(ring, pending, capture, home, line.trim_end());
    writeln!(stream, "{reply}")?;
    stream.flush()?;
    Ok(())
}

fn serve(mut ring: Ring, mut capture: Option<LiveCapture>, home: &Path) {
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
        let reply = dispatch(&mut ring, &mut pending, capture.as_mut(), home, line.trim_end());
        println!("{reply}");
        let _ = std::io::stdout().flush();
    }
}

fn dispatch(
    ring: &mut Ring,
    pending: &mut Vec<PlashkaEvent>,
    mut capture: Option<&mut LiveCapture>,
    home: &Path,
    line: &str,
) -> String {
    let (command, arg) = line.split_once('\t').unwrap_or((line, ""));
    if command == "bind" {
        if let Some(ref mut capture) = capture {
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
        "encoder" | "bitrate" | "length" | "share" | "place" => {
            let mut plan = read_plan(home);
            let accepted = match command {
                "encoder" => {
                    plan.codec = recorder_codec(arg).to_string();
                    true
                }
                "bitrate" => bitrate_mbit(arg).is_some_and(|n| {
                    plan.bitrate_mbit = n;
                    true
                }),
                "length" => length_secs(arg).is_some_and(|n| {
                    plan.length_secs = n;
                    true
                }),
                "share" => share_percent(arg).is_some_and(|n| {
                    plan.share = n;
                    true
                }),
                "place" => storage_name(arg).is_some_and(|n| {
                    plan.storage = n.to_string();
                    true
                }),
                _ => false,
            };
            if !accepted {
                return "none".to_string();
            }
            write_plan(home, &plan);
            if let Some(capture) = capture {
                capture.apply(ring, plan);
            }
            "ok".to_string()
        }
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

#[cfg(test)]
mod tests {
    use super::{
        even_side, frame_in_monitors_json, recorder_args, recorder_codec, RecordPlan,
    };
    use std::path::Path;

    #[test]
    fn the_recorder_hears_hevc_and_av1() {
        assert_eq!(recorder_codec("HEVC"), "hevc");
        assert_eq!(recorder_codec("hevc"), "hevc");
        assert_eq!(recorder_codec("AV1"), "av1");
        assert_eq!(recorder_codec("av1"), "av1");
        assert_eq!(recorder_codec("H.264"), "h264");
        assert_eq!(recorder_codec("nope"), "h264");
    }

    #[test]
    fn the_knobs_reach_the_recorder() {
        let mut plan = RecordPlan::standard();
        plan.bitrate_mbit = 50;
        plan.length_secs = 120;
        plan.share = 50;
        plan.storage = "disk".to_string();
        plan.codec = "av1".to_string();
        let args = recorder_args(
            "HDMI-A-1",
            &plan,
            Some((1920, 1080)),
            Path::new("/tmp/replay"),
            Path::new("/tmp/replay.sock"),
        );
        let joined = args.join(" ");
        assert!(joined.contains("-r 120"), "{joined}");
        assert!(joined.contains("-replay-storage disk"), "{joined}");
        assert!(joined.contains("-k av1"), "{joined}");
        assert!(joined.contains("-bm cbr"), "{joined}");
        assert!(joined.contains("-q 50000"), "{joined}");
        assert!(joined.contains("-s 960x540"), "{joined}");

        plan.share = 100;
        let full = recorder_args(
            "HDMI-A-1",
            &plan,
            Some((1920, 1080)),
            Path::new("/tmp/replay"),
            Path::new("/tmp/replay.sock"),
        )
        .join(" ");
        assert!(!full.contains(" -s "), "{full}");
        assert_eq!(even_side(1920, 80), 1536);
        assert_eq!(even_side(1080, 80), 864);
    }

    #[test]
    fn the_capture_screen_is_read_from_the_monitor_list() {
        let json = r#"[{"name": "HDMI-A-1", "description": "x", "width": 1920, "height": 1080}, {"name": "DP-1", "width": 1680, "height": 1050}]"#;
        assert_eq!(frame_in_monitors_json(json, "HDMI-A-1"), Some((1920, 1080)));
        assert_eq!(frame_in_monitors_json(json, "DP-1"), Some((1680, 1050)));
        assert_eq!(frame_in_monitors_json(json, "gone"), None);
    }
}

fn parse_bind_day(text: &str) -> BindDay {
    let mut parts = text.split('-');
    let year = parts.next().unwrap_or("1970").parse().unwrap_or(1970);
    let month = parts.next().unwrap_or("1").parse().unwrap_or(1);
    let day = parts.next().unwrap_or("1").parse().unwrap_or(1);
    BindDay::new(year, month, day)
}
