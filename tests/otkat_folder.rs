use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use otkati::{BindDay, PlashkaEvent, Ring};

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
        let path = std::env::temp_dir().join(format!("otkati-otkat-folder-{tick}-{n}"));
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
fn the_next_otkat_is_written_in_the_folder_the_face_handed() {
    let old = OtkatFolder::new();
    fs::write(old.path.join("otkat5-2026-10-24.mp4"), b"kept").unwrap();
    let scratch = OtkatFolder::new();
    let handed = scratch.path.join("next");
    let mut ring = Ring::with_clock(old.path.clone(), || BindDay::new(2026, 10, 25));
    ring.hold(Duration::from_secs(120));

    ring.take_otkat_folder(&handed);

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.name(), "otkat1-2026-10-25.mp4");
    assert!(
        !handed.exists(),
        "the folder appears with the file, after the plashka"
    );
    drop(event);

    assert_eq!(mp4_duration_secs(&handed.join("otkat1-2026-10-25.mp4")), 120);
    assert!(!old.path.join("otkat1-2026-10-25.mp4").exists());
    assert!(!old.path.join("otkat6-2026-10-25.mp4").exists());
}

#[test]
fn files_already_in_the_old_folder_stay_there() {
    let old = OtkatFolder::new();
    let kept = old.path.join("otkat5-2026-10-24.mp4");
    fs::write(&kept, b"kept").unwrap();
    let scratch = OtkatFolder::new();
    let handed = scratch.path.join("next");
    let mut ring = Ring::with_clock(old.path.clone(), || BindDay::new(2026, 10, 25));
    ring.hold(Duration::from_secs(120));

    ring.take_otkat_folder(&handed);
    let event = ring.bind(Ring::default_bind());
    drop(event);

    assert_eq!(fs::read(&kept).unwrap(), b"kept");
    assert!(!handed.join("otkat5-2026-10-24.mp4").exists());
}
