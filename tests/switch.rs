use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use otkati::{PlashkaEvent, Ring, Switch};

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

#[test]
fn switching_off_stops_the_ring_and_leaves_the_program_running() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::new(folder.path.clone());
    ring.hold(std::time::Duration::from_secs(120));

    ring.take_switch(Switch::Off);

    let event = ring.bind(Ring::default_bind());

    assert_eq!(event, Some(PlashkaEvent::RingOff));
    let written: Vec<_> = fs::read_dir(&folder.path).unwrap().collect();
    assert!(
        written.is_empty(),
        "a bind while the ring is off must not put a file in the otkat folder"
    );
}
