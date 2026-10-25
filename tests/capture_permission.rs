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
        let path = std::env::temp_dir().join(format!("otkati-capture-{tick}-{n}"));
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
fn bind_before_the_screen_answer_reports_no_access_and_writes_no_file() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::enter_session(folder.path.clone(), None);

    let event = ring.bind();

    assert_eq!(event, PlashkaEvent::NoAccess);
    let written: Vec<_> = fs::read_dir(&folder.path).unwrap().collect();
    assert!(
        written.is_empty(),
        "a ring with no screen answer must not put a file in the otkat folder"
    );
}
