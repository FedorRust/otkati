use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use otkati::{BindDay, PlashkaEvent, Ring, SessionAutostart};

fn ring_program_dir() -> PathBuf {
    let test_exe = std::env::current_exe().expect("test executable");
    test_exe
        .parent()
        .and_then(|deps| deps.parent())
        .expect("the ring program lives beside the test executable")
        .to_path_buf()
}

#[test]
fn after_hyprland_login_the_ring_is_already_running() {
    let mut session = SessionAutostart::shipped().log_in(ring_program_dir());

    assert!(
        session.ring_is_running(),
        "after login the ring is already running, before anyone opens it"
    );
}

#[test]
fn restarting_serpantinum_does_not_stop_the_ring() {
    let mut session = SessionAutostart::shipped().log_in(ring_program_dir());
    let ring = session.ring_pid();
    assert!(
        !session.ring_is_child_of_shell(),
        "the ring is not a child of Serpantinum"
    );

    session.restart_shell();

    assert!(
        session.ring_is_running(),
        "restarting Serpantinum leaves the ring process running"
    );
    assert_eq!(session.ring_pid(), ring);
}

struct OtkatFolder {
    path: PathBuf,
}

impl OtkatFolder {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("otkati-session-life-{tick}-{n}"));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for OtkatFolder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn mp4_duration_secs(path: &Path) -> u64 {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=nw=1:nk=1")
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ffprobe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    let secs: f64 = text.trim().parse().unwrap();
    secs.round() as u64
}

#[test]
fn minutes_the_ring_already_holds_still_save_after_serpantinum_restarts() {
    let folder = OtkatFolder::new();
    let mut session = SessionAutostart::shipped().log_in_with_clock(
        folder.path.clone(),
        ring_program_dir(),
        || BindDay::new(2026, 10, 25),
    );
    session.ring().hold(Duration::from_secs(120));

    session.restart_shell();

    assert!(
        session.ring_is_running(),
        "the ring process is still there to save from"
    );
    let event = session.ring().bind(Ring::default_bind());

    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.name(), "otkat1-2026-10-25.mp4");
    assert_eq!(saved.duration(), Duration::from_secs(120));
    let path = folder.path.join("otkat1-2026-10-25.mp4");
    assert!(
        !path.exists(),
        "the success plashka comes out before the file is in the folder"
    );
    drop(event);

    assert_eq!(mp4_duration_secs(&path), 120);
}
