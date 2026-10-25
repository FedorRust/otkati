use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

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
