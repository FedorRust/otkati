use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use otkati::{PlashkaEvent, Ring};

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
        let path = std::env::temp_dir().join(format!("otkati-empty-ring-{tick}-{n}"));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for OtkatFolder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn bind_on_an_empty_ring_yields_a_plashka_and_writes_no_otkat() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::new(folder.path.clone());

    let event = ring.bind(Ring::default_bind());

    assert_eq!(event, Some(PlashkaEvent::NothingToSave));
    let written: Vec<_> = fs::read_dir(&folder.path).unwrap().collect();
    assert!(
        written.is_empty(),
        "an empty ring must not put a file in the otkat folder"
    );
}

#[test]
fn the_default_bind_is_alt_z() {
    assert_eq!(Ring::default_bind().text(), "Alt+Z");
}
