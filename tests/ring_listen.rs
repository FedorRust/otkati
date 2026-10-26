use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn ring_program() -> PathBuf {
    let test_exe = std::env::current_exe().expect("test executable");
    test_exe
        .parent()
        .and_then(|deps| deps.parent())
        .expect("the ring program lives beside the test executable")
        .join("otkati-ring")
}

struct HeardRing {
    child: Child,
    folder: PathBuf,
    socket: PathBuf,
}

impl HeardRing {
    fn start() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder = std::env::temp_dir().join(format!("otkati-listen-{tick}-{n}"));
        let socket = std::env::temp_dir().join(format!("otkati-listen-{n}.sock"));
        fs::create_dir_all(&folder).unwrap();
        let _ = fs::remove_file(&socket);
        let child = Command::new(ring_program())
            .env("OTKATI_SERVE", "1")
            .env("OTKATI_SOCKET", &socket)
            .env("OTKATI_OTKAT_FOLDER", &folder)
            .env("OTKATI_BIND_DAY", "2026-10-25")
            .env("OTKATI_READY", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("the ring process starts");
        Self {
            child,
            folder,
            socket,
        }
    }

    fn running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

impl Drop for HeardRing {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.folder);
        let _ = fs::remove_file(&self.socket);
    }
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
    reply
}

#[test]
fn the_ring_stays_up_with_stdin_closed_and_a_bind_saves_what_it_holds() {
    let mut ring = HeardRing::start();
    let socket = ring.socket.clone();
    let mut connected = false;
    for _ in 0..50 {
        if UnixStream::connect(&socket).is_ok() {
            connected = true;
            break;
        }
        if !ring.running() {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        connected && ring.running(),
        "a served ring with nothing on stdin stays up and listens"
    );

    say(&socket, "hold\t15");
    say(&socket, "bind\tAlt+Z");
    say(&socket, "commit");

    let path = ring.folder.join("otkat1-2026-10-25.mp4");
    assert!(
        path.is_file(),
        "a bind saves the minutes the ring is holding"
    );
    assert!(
        ring.running(),
        "saving an otkat leaves the ring process running"
    );
}
