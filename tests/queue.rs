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
        let path = std::env::temp_dir().join(format!("otkati-queue-{tick}-{n}"));
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
fn a_second_bind_before_the_first_file_lands_keeps_both_moments() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.hold(Duration::from_secs(120));

    let first = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &first else {
        panic!("expected a saved otkat, got {first:?}");
    };
    assert_eq!(saved.name(), "otkat1-2026-10-25.mp4");
    assert_eq!(saved.duration(), Duration::from_secs(120));
    let first_path = folder.path.join("otkat1-2026-10-25.mp4");
    assert!(
        !first_path.exists(),
        "the success plashka comes out before the file is in the folder"
    );

    ring.hold(Duration::from_secs(45));
    let second = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &second else {
        panic!("expected a saved otkat, got {second:?}");
    };
    assert_eq!(saved.name(), "otkat2-2026-10-25.mp4");
    assert_eq!(saved.duration(), Duration::from_secs(45));
    let second_path = folder.path.join("otkat2-2026-10-25.mp4");
    assert!(
        !second_path.exists(),
        "the success plashka comes out before the file is in the folder"
    );
    assert!(!first_path.exists());

    drop(first);
    assert_eq!(mp4_duration_secs(&first_path), 120);
    assert!(!second_path.exists(), "the queue writes one file at a time");

    drop(second);
    assert_eq!(mp4_duration_secs(&second_path), 45);
    assert_eq!(mp4_duration_secs(&first_path), 120);
}

#[test]
fn two_otkats_in_the_queue_take_adjacent_numbers() {
    let folder = OtkatFolder::new();
    fs::write(folder.path.join("otkat2-2026-10-24.mp4"), b"kept").unwrap();
    fs::write(folder.path.join("otkat10-2026-10-24.mp4"), b"kept").unwrap();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.hold(Duration::from_secs(30));

    let first = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &first else {
        panic!("expected a saved otkat, got {first:?}");
    };
    assert_eq!(saved.name(), "otkat11-2026-10-25.mp4");
    assert_eq!(saved.duration(), Duration::from_secs(30));

    ring.hold(Duration::from_secs(15));
    let second = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &second else {
        panic!("expected a saved otkat, got {second:?}");
    };
    assert_eq!(saved.name(), "otkat12-2026-10-25.mp4");
    assert_eq!(saved.duration(), Duration::from_secs(15));
    assert!(!folder.path.join("otkat11-2026-10-25.mp4").exists());
    assert!(!folder.path.join("otkat12-2026-10-25.mp4").exists());

    drop(first);
    drop(second);

    assert!(folder.path.join("otkat2-2026-10-24.mp4").is_file());
    assert!(folder.path.join("otkat10-2026-10-24.mp4").is_file());
    assert!(!folder.path.join("otkat3-2026-10-25.mp4").exists());
    assert_eq!(
        mp4_duration_secs(&folder.path.join("otkat11-2026-10-25.mp4")),
        30
    );
    assert_eq!(
        mp4_duration_secs(&folder.path.join("otkat12-2026-10-25.mp4")),
        15
    );
}
