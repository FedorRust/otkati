use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use otkati::{MonitorMoment, Ring, Screen};

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
        let path = std::env::temp_dir().join(format!("otkati-ring-monitor-{tick}-{n}"));
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
fn choosing_a_monitor_in_settings_moves_the_ring_to_it() {
    let folder = OtkatFolder::new();
    let desk = Screen::new("desk");
    let laptop = Screen::new("laptop");
    let mut ring = Ring::enter_session(folder.path.clone(), Some(desk.clone()));
    ring.take_monitor(MonitorMoment::new(desk.clone(), Duration::from_secs(2)));

    ring.take_ring_monitor(laptop.clone());

    assert_eq!(ring.monitor(), Some(&laptop));
    ring.take_monitor(MonitorMoment::new(desk, Duration::from_secs(5)));
    let segment = ring.segment().expect("the ring keeps what it already held");
    assert_eq!(segment.monitor(), &laptop);
    assert_eq!(segment.duration(), Duration::from_secs(2));
    ring.take_monitor(MonitorMoment::new(laptop.clone(), Duration::from_secs(3)));
    let segment = ring.segment().expect("the chosen monitor entered the ring");
    assert_eq!(segment.monitor(), &laptop);
    assert_eq!(segment.duration(), Duration::from_secs(5));
    let written: Vec<_> = fs::read_dir(&folder.path).unwrap().collect();
    assert!(
        written.is_empty(),
        "changing the ring monitor must not put a file in the otkat folder"
    );
}
