use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use otkati::{BindDay, MonitorMoment, PlashkaEvent, Ring, Screen};

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
        let path = std::env::temp_dir().join(format!("otkati-length-{tick}-{n}"));
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
fn an_otkat_is_not_longer_than_the_chosen_length() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.take_length(Duration::from_secs(15));
    ring.hold(Duration::from_secs(120));

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.duration(), Duration::from_secs(15));
    drop(event);

    assert_eq!(
        mp4_duration_secs(&folder.path.join("otkat1-2026-10-25.mp4")),
        15
    );
}

#[test]
fn an_unfilled_ring_saves_what_it_holds_without_empty_seconds() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.take_length(Duration::from_secs(1200));
    ring.hold(Duration::from_secs(601));

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.duration(), Duration::from_secs(601));
    drop(event);

    assert_eq!(
        mp4_duration_secs(&folder.path.join("otkat1-2026-10-25.mp4")),
        601
    );
}

#[test]
fn a_length_shorter_than_fifteen_seconds_does_not_change_the_aim() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.take_length(Duration::from_secs(15));
    ring.take_length(Duration::from_secs(14));
    ring.hold(Duration::from_secs(120));

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.duration(), Duration::from_secs(15));
    drop(event);

    assert_eq!(
        mp4_duration_secs(&folder.path.join("otkat1-2026-10-25.mp4")),
        15
    );
}

#[test]
fn a_length_past_twenty_minutes_does_not_change_the_aim() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.take_length(Duration::from_secs(1200));
    ring.take_length(Duration::from_secs(1201));
    ring.hold(Duration::from_secs(1300));

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.duration(), Duration::from_secs(1200));
    drop(event);

    assert_eq!(
        mp4_duration_secs(&folder.path.join("otkat1-2026-10-25.mp4")),
        1200
    );
}

#[test]
fn a_fraction_of_a_second_does_not_change_the_aim() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.take_length(Duration::from_secs(90));
    ring.take_length(Duration::from_millis(91_500));
    ring.hold(Duration::from_secs(120));

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.duration(), Duration::from_secs(90));
    drop(event);

    assert_eq!(
        mp4_duration_secs(&folder.path.join("otkat1-2026-10-25.mp4")),
        90
    );
}

#[test]
fn a_later_bind_saves_the_tail_not_a_new_clip_of_the_old_head() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    let screen = Screen::new("desk");
    ring.remember_screen(screen.clone());
    ring.take_length(Duration::from_secs(15));
    ring.take_monitor(
        MonitorMoment::new(screen.clone(), Duration::from_secs(10))
            .picture(200, 0, 0)
            .cursor(false),
    );
    ring.take_monitor(
        MonitorMoment::new(screen, Duration::from_secs(10))
            .picture(0, 0, 200)
            .cursor(false),
    );

    assert_eq!(
        ring.segment()
            .expect("the ring keeps the newest moments")
            .duration(),
        Duration::from_secs(15)
    );

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.duration(), Duration::from_secs(15));
    let path = folder.path.join(saved.name());
    drop(event);

    let early = pixel_at(&path, 1.0);
    let later = pixel_at(&path, 6.0);
    assert!(
        early.0 > 150 && early.2 < 40,
        "the older moment is only the start of the tail, got {early:?}"
    );
    assert!(
        later.2 > 150 && later.0 < 40,
        "a later moment is what the bind saves, got {later:?}"
    );
}

fn pixel_at(path: &Path, at_secs: f64) -> (u8, u8, u8) {
    let output = Command::new("ffmpeg")
        .arg("-v")
        .arg("error")
        .arg("-ss")
        .arg(at_secs.to_string())
        .arg("-i")
        .arg(path)
        .arg("-frames:v")
        .arg("1")
        .arg("-f")
        .arg("rawvideo")
        .arg("-pix_fmt")
        .arg("rgb24")
        .arg("-")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ffmpeg failed to read a frame: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.len() >= 3);
    (output.stdout[0], output.stdout[1], output.stdout[2])
}

#[test]
fn changing_the_length_does_not_rewrite_a_saved_otkat() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.hold(Duration::from_secs(120));

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.name(), "otkat1-2026-10-25.mp4");
    assert_eq!(saved.duration(), Duration::from_secs(120));
    ring.take_length(Duration::from_secs(15));
    drop(event);

    let path = folder.path.join("otkat1-2026-10-25.mp4");
    assert_eq!(mp4_duration_secs(&path), 120);
    let before = fs::read(&path).unwrap();
    ring.take_length(Duration::from_secs(30));
    assert_eq!(fs::read(&path).unwrap(), before);

    let mut names: Vec<_> = fs::read_dir(&folder.path)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![std::ffi::OsString::from("otkat1-2026-10-25.mp4")]
    );
}
