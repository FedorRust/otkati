use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use otkati::{Ring, RingPlace};

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
fn the_ring_folder_is_not_the_otkat_folder() {
    let home = std::env::var_os("HOME").expect("HOME");
    assert_eq!(
        Ring::default_ring_folder(),
        PathBuf::from(&home).join(".local/state/otkati/ring")
    );
    assert_ne!(Ring::default_ring_folder(), Ring::default_otkat_folder());
}
