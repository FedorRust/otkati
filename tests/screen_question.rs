use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct AskedRing {
    child: Child,
    socket: PathBuf,
}

impl AskedRing {
    fn start(home: &PathBuf, socket: &PathBuf, ask: bool, asker: Option<&PathBuf>) -> Self {
        let _ = fs::remove_file(socket);
        let mut command = Command::new(ring_program());
        command
            .env("OTKATI_SERVE", "1")
            .env("OTKATI_SOCKET", socket)
            .env("OTKATI_HOME", home)
            .env("OTKATI_OTKAT_FOLDER", home.join("otkats"))
            .env("OTKATI_BIND_DAY", "2026-10-25")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        if ask {
            command.env("OTKATI_ASK_SCREEN", "1");
        }
        if let Some(asker) = asker {
            command.env("OTKATI_SCREEN_ASK", asker);
        }
        let child = command.spawn().expect("the ring process starts");
        Self {
            child,
            socket: socket.clone(),
        }
    }

    fn running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

impl Drop for AskedRing {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_file(&self.socket);
    }
}

fn ring_program() -> PathBuf {
    let test_exe = std::env::current_exe().expect("test executable");
    test_exe
        .parent()
        .and_then(|deps| deps.parent())
        .expect("the ring program lives beside the test executable")
        .join("otkati-ring")
}

fn say(socket: &std::path::Path, line: &str) -> String {
    let mut stream = UnixStream::connect(socket).expect("the ring is listening");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    writeln!(stream, "{line}").unwrap();
    stream.flush().unwrap();
    let mut reply = String::new();
    stream.read_to_string(&mut reply).unwrap();
    reply.trim().to_string()
}

fn wait_until_listening(ring: &mut AskedRing) -> bool {
    for _ in 0..50 {
        if UnixStream::connect(&ring.socket).is_ok() && ring.running() {
            return true;
        }
        if !ring.running() {
            return false;
        }
        thread::sleep(Duration::from_millis(20));
    }
    false
}

#[test]
fn the_system_answer_is_remembered_and_the_next_start_does_not_ask() {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tick = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let home = std::env::temp_dir().join(format!("otkati-ask-{tick}-{n}"));
    let socket = std::env::temp_dir().join(format!("otkati-ask-{n}.sock"));
    fs::create_dir_all(&home).unwrap();
    let asker = home.join("answer-desk");
    fs::write(&asker, "#!/bin/sh\nprintf '%s\\n' desk\n").unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&asker, fs::Permissions::from_mode(0o755)).unwrap();

    let mut first = AskedRing::start(&home, &socket, true, Some(&asker));
    assert!(
        wait_until_listening(&mut first),
        "the ring stays up after the system answers"
    );
    assert_eq!(
        say(&socket, "query"),
        "0\tdesk\t0",
        "the answered screen is the ring monitor"
    );
    let answer = fs::read_to_string(home.join(".local/state/otkati/screen-answer"))
        .expect("the screen answer is remembered on disk");
    assert_eq!(answer.lines().next(), Some("desk"));
    drop(first);

    let mut next = AskedRing::start(&home, &socket, false, None);
    assert!(
        wait_until_listening(&mut next),
        "the next start does not wait on the system question"
    );
    assert_eq!(
        say(&socket, "query"),
        "0\tdesk\t0",
        "the next session already knows the screen and does not ask again"
    );

    let _ = fs::remove_dir_all(&home);
}
