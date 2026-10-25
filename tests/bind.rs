use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use otkati::{Bind, Ring};

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
        let path = std::env::temp_dir().join(format!("otkati-bind-{tick}-{n}"));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for OtkatFolder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn folder_is_empty(folder: &OtkatFolder) -> bool {
    fs::read_dir(&folder.path).unwrap().next().is_none()
}

#[test]
fn a_chord_other_than_alt_z_does_not_save() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::new(folder.path.clone());

    let event = ring.bind(Bind::new("Ctrl+Shift+F9"));

    assert_eq!(event, None);
    assert!(
        folder_is_empty(&folder),
        "a chord that is not the bind must not put a file in the otkat folder"
    );
}
