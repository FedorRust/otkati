use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use otkati::{PlashkaEvent, Ring, RingPlace};

struct Home {
    path: PathBuf,
}

impl Home {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("otkati-ring-home-{tick}-{n}"));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn the_ring_place_starts_in_memory() {
    let home = Home::new();
    let otkats = home.path.join("Videos").join("Откаты");
    let ring = Ring::with_home(&otkats, &home.path);

    assert_eq!(ring.place(), RingPlace::Memory);
    assert!(ring.ring_folder().is_none());
    assert!(
        !home.path.join(".local/state/otkati/ring").exists(),
        "memory does not create the ring folder"
    );
    assert!(
        !otkats.exists(),
        "a new ring does not create the otkat folder"
    );
}

#[test]
fn choosing_disk_puts_the_ring_in_the_ring_folder() {
    let home = Home::new();
    let otkats = home.path.join("Videos").join("Откаты");
    let mut ring = Ring::with_home(&otkats, &home.path);
    ring.hold(Duration::from_secs(120));

    ring.take_place(RingPlace::Disk);

    let ring_folder = home.path.join(".local/state/otkati/ring");
    assert_eq!(ring.place(), RingPlace::Disk);
    assert_eq!(ring.ring_folder(), Some(ring_folder.as_path()));
    assert!(ring_folder.is_dir());
    assert_ne!(ring_folder, otkats);
    assert!(
        !otkats.exists(),
        "choosing disk must not write a file in the otkat folder"
    );
    let copied: Vec<_> = fs::read_dir(&ring_folder).unwrap().collect();
    assert!(
        copied.is_empty(),
        "minutes already held are not copied into the ring folder"
    );
}

#[test]
fn changing_the_place_starts_the_ring_over_without_an_otkat() {
    let home = Home::new();
    let otkats = home.path.join("Videos").join("Откаты");
    let mut ring = Ring::with_home(&otkats, &home.path);
    ring.hold(Duration::from_secs(120));

    ring.take_place(RingPlace::Disk);

    assert!(
        !otkats.exists(),
        "changing the place must not write a file in the otkat folder"
    );
    assert_eq!(
        ring.bind(Ring::default_bind()),
        Some(PlashkaEvent::NothingToSave)
    );
    assert!(
        !otkats.exists(),
        "minutes already held are not saved after the place changes"
    );

    ring.hold(Duration::from_secs(45));
    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected the minutes held after the change, got {event:?}");
    };
    assert_eq!(saved.duration(), Duration::from_secs(45));
}

#[test]
fn changing_back_to_memory_starts_the_ring_over_without_an_otkat() {
    let home = Home::new();
    let otkats = home.path.join("Videos").join("Откаты");
    let mut ring = Ring::with_home(&otkats, &home.path);
    ring.take_place(RingPlace::Disk);
    ring.hold(Duration::from_secs(120));

    ring.take_place(RingPlace::Memory);

    assert_eq!(ring.place(), RingPlace::Memory);
    assert!(ring.ring_folder().is_none());
    assert!(
        !otkats.exists(),
        "changing back to memory must not write a file in the otkat folder"
    );
    assert_eq!(
        ring.bind(Ring::default_bind()),
        Some(PlashkaEvent::NothingToSave)
    );

    ring.hold(Duration::from_secs(15));
    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected the minutes held after the change, got {event:?}");
    };
    assert_eq!(saved.duration(), Duration::from_secs(15));
}

#[test]
fn handing_the_current_place_keeps_minutes_already_held() {
    let home = Home::new();
    let otkats = home.path.join("Videos").join("Откаты");
    let mut ring = Ring::with_home(&otkats, &home.path);
    ring.hold(Duration::from_secs(120));

    ring.take_place(RingPlace::Memory);

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected the minutes already held, got {event:?}");
    };
    assert_eq!(saved.duration(), Duration::from_secs(120));
    drop(event);

    ring.take_place(RingPlace::Disk);
    ring.hold(Duration::from_secs(90));
    ring.take_place(RingPlace::Disk);

    assert!(ring.ring_folder().unwrap().is_dir());
    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected the minutes kept on disk, got {event:?}");
    };
    assert_eq!(saved.duration(), Duration::from_secs(90));
}

#[test]
fn choosing_disk_keeps_the_encoded_ring_in_the_ring_folder() {
    let home = Home::new();
    let otkats = home.path.join("Videos").join("Откаты");
    let mut ring = Ring::with_home(&otkats, &home.path);
    ring.take_place(RingPlace::Disk);

    ring.hold(Duration::from_secs(30));

    let segment = home
        .path
        .join(".local/state/otkati/ring")
        .join("segment.mp4");
    assert!(
        segment.is_file(),
        "choosing disk puts the encoded ring in the ring folder"
    );
    assert_eq!(mp4_duration_secs(&segment), 30);
    assert!(
        !otkats.exists(),
        "the encoded ring is not an otkat"
    );
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
fn the_ring_folder_is_not_the_otkat_folder() {
    let home = std::env::var_os("HOME").expect("HOME");
    assert_eq!(
        Ring::default_ring_folder(),
        PathBuf::from(&home).join(".local/state/otkati/ring")
    );
    assert_ne!(Ring::default_ring_folder(), Ring::default_otkat_folder());
}
