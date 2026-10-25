use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use otkati::{BindDay, PlashkaEvent, Ring, Switch};

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
        let path = std::env::temp_dir().join(format!("otkati-switch-{tick}-{n}"));
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
fn switching_off_stops_the_ring_and_leaves_the_program_running() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::new(folder.path.clone());
    ring.hold(Duration::from_secs(120));

    ring.take_switch(Switch::Off);

    let event = ring.bind(Ring::default_bind());

    assert_eq!(event, Some(PlashkaEvent::RingOff));
    let written: Vec<_> = fs::read_dir(&folder.path).unwrap().collect();
    assert!(
        written.is_empty(),
        "a bind while the ring is off must not put a file in the otkat folder"
    );
}

#[test]
fn turning_the_ring_on_starts_over_without_the_minutes_it_was_off() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.hold(Duration::from_secs(120));
    ring.take_switch(Switch::Off);
    ring.hold(Duration::from_secs(300));
    ring.take_switch(Switch::On);

    let idle = ring.bind(Ring::default_bind());
    assert_eq!(idle, Some(PlashkaEvent::NothingToSave));
    let written: Vec<_> = fs::read_dir(&folder.path).unwrap().collect();
    assert!(
        written.is_empty(),
        "minutes from before the switch and while the ring was off must not be saved"
    );

    ring.hold(Duration::from_secs(45));
    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected only the minutes gathered after the ring came back on, got {event:?}");
    };
    assert_eq!(saved.name(), "otkat1-2026-10-25.mp4");
    assert_eq!(saved.duration(), Duration::from_secs(45));
    drop(event);

    assert_eq!(
        mp4_duration_secs(&folder.path.join("otkat1-2026-10-25.mp4")),
        45
    );
}
