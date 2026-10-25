use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
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
        let path = std::env::temp_dir().join(format!("otkati-first-otkat-{tick}-{n}"));
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
fn two_minutes_held_save_otkat1_for_the_bind_day() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.hold(Duration::from_secs(120));

    let event = ring.bind();

    let PlashkaEvent::Saved(saved) = &event else {
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
    let names: Vec<_> = fs::read_dir(&folder.path)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(
        names,
        vec![std::ffi::OsString::from("otkat1-2026-10-25.mp4")]
    );
}

#[test]
fn a_missing_number_in_the_folder_is_not_reused() {
    let folder = OtkatFolder::new();
    fs::write(folder.path.join("otkat1-2026-10-20.mp4"), b"kept").unwrap();
    fs::write(folder.path.join("otkat3-2026-10-24.mp4"), b"kept").unwrap();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.hold(Duration::from_secs(120));

    let event = ring.bind();

    let PlashkaEvent::Saved(saved) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.name(), "otkat4-2026-10-25.mp4");
    drop(event);

    assert!(folder.path.join("otkat1-2026-10-20.mp4").is_file());
    assert!(folder.path.join("otkat3-2026-10-24.mp4").is_file());
    assert!(folder.path.join("otkat4-2026-10-25.mp4").is_file());
    assert!(!folder.path.join("otkat2-2026-10-25.mp4").exists());
}

#[test]
fn the_next_number_follows_the_largest_even_past_nine() {
    let folder = OtkatFolder::new();
    fs::write(folder.path.join("otkat2-2026-10-24.mp4"), b"kept").unwrap();
    fs::write(folder.path.join("otkat10-2026-10-24.mp4"), b"kept").unwrap();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.hold(Duration::from_secs(120));

    let event = ring.bind();

    let PlashkaEvent::Saved(saved) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.name(), "otkat11-2026-10-25.mp4");
    drop(event);
    assert!(folder.path.join("otkat10-2026-10-24.mp4").is_file());
    assert!(folder.path.join("otkat11-2026-10-25.mp4").is_file());
}

#[test]
fn the_otkat_number_does_not_restart_on_a_new_day() {
    let folder = OtkatFolder::new();
    fs::write(folder.path.join("otkat5-2026-10-24.mp4"), b"kept").unwrap();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.hold(Duration::from_secs(120));

    let event = ring.bind();

    let PlashkaEvent::Saved(saved) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.name(), "otkat6-2026-10-25.mp4");
    drop(event);

    assert!(folder.path.join("otkat5-2026-10-24.mp4").is_file());
    assert!(folder.path.join("otkat6-2026-10-25.mp4").is_file());
    assert!(!folder.path.join("otkat1-2026-10-25.mp4").exists());
}

#[test]
fn the_file_keeps_the_bind_day_when_the_write_finishes_after_midnight() {
    let folder = OtkatFolder::new();
    let day = Arc::new(AtomicU8::new(25));
    let clock_day = Arc::clone(&day);
    let mut ring = Ring::with_clock(folder.path.clone(), move || {
        BindDay::new(2026, 10, clock_day.load(Ordering::SeqCst))
    });
    ring.hold(Duration::from_secs(120));

    let event = ring.bind();
    let PlashkaEvent::Saved(saved) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.name(), "otkat1-2026-10-25.mp4");
    day.store(26, Ordering::SeqCst);
    drop(event);

    assert!(folder.path.join("otkat1-2026-10-25.mp4").is_file());
    assert!(!folder.path.join("otkat1-2026-10-26.mp4").exists());
}

#[test]
fn the_ring_keeps_turning_and_the_next_bind_writes_another_file() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.hold(Duration::from_secs(120));

    let first = ring.bind();
    let PlashkaEvent::Saved(saved) = &first else {
        panic!("expected a saved otkat, got {first:?}");
    };
    assert_eq!(saved.name(), "otkat1-2026-10-25.mp4");
    assert_eq!(saved.duration(), Duration::from_secs(120));
    drop(first);

    let second = ring.bind();
    let PlashkaEvent::Saved(saved) = &second else {
        panic!("expected the ring to still be turning, got {second:?}");
    };
    assert_eq!(saved.name(), "otkat2-2026-10-25.mp4");
    assert_eq!(saved.duration(), Duration::from_secs(120));
    drop(second);

    assert_eq!(
        mp4_duration_secs(&folder.path.join("otkat1-2026-10-25.mp4")),
        120
    );
    assert_eq!(
        mp4_duration_secs(&folder.path.join("otkat2-2026-10-25.mp4")),
        120
    );
}

#[test]
fn a_ring_past_the_chosen_length_saves_ten_minutes() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.hold(Duration::from_secs(720));

    let event = ring.bind();
    let PlashkaEvent::Saved(saved) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.duration(), Duration::from_secs(600));
    drop(event);

    assert_eq!(
        mp4_duration_secs(&folder.path.join("otkat1-2026-10-25.mp4")),
        600
    );
}

#[test]
fn the_otkat_folder_is_created_when_the_file_is_written() {
    let parent = OtkatFolder::new();
    let path = parent.path.join("Откаты");
    assert!(!path.exists());
    let mut ring = Ring::with_clock(&path, || BindDay::new(2026, 10, 25));
    ring.hold(Duration::from_secs(120));

    let event = ring.bind();
    assert!(
        !path.exists(),
        "the folder appears with the file, after the plashka"
    );
    drop(event);

    let file = path.join("otkat1-2026-10-25.mp4");
    assert_eq!(mp4_duration_secs(&file), 120);
}

#[test]
fn the_default_otkat_folder_is_videos_otkaty() {
    let home = std::env::var_os("HOME").expect("HOME");
    assert_eq!(
        Ring::default_otkat_folder(),
        PathBuf::from(home).join("Videos").join("Откаты")
    );
}

#[test]
fn an_empty_ring_does_not_create_the_otkat_folder() {
    let parent = OtkatFolder::new();
    let path = parent.path.join("Откаты");
    let mut ring = Ring::new(&path);

    let event = ring.bind();

    assert_eq!(event, PlashkaEvent::NothingToSave);
    assert!(!path.exists());
}
