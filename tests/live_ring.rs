use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use otkati::{MonitorMoment, PlashkaEvent, Ring, Screen};

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
        let path = std::env::temp_dir().join(format!("otkati-live-ring-{tick}-{n}"));
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
fn after_the_screen_answer_the_chosen_monitor_enters_the_ring() {
    let folder = OtkatFolder::new();
    let screen = Screen::new("Alexey's screen");
    let mut ring = Ring::enter_session(folder.path.clone(), None);
    ring.remember_screen(screen.clone());

    ring.take_monitor(MonitorMoment::new(screen.clone(), Duration::from_secs(2)));

    let segment = ring
        .segment()
        .expect("the chosen monitor entered the ring");
    assert_eq!(segment.monitor(), &screen);
    assert_eq!(segment.duration(), Duration::from_secs(2));
    let written: Vec<_> = fs::read_dir(&folder.path).unwrap().collect();
    assert!(
        written.is_empty(),
        "the monitor enters the ring, not the otkat folder"
    );
}

#[test]
fn a_monitor_the_person_did_not_choose_stays_out_of_the_ring() {
    let folder = OtkatFolder::new();
    let chosen = Screen::new("Alexey's screen");
    let other = Screen::new("The other screen");
    let mut ring = Ring::enter_session(folder.path.clone(), Some(chosen.clone()));

    ring.take_monitor(MonitorMoment::new(other, Duration::from_secs(2)));

    assert!(ring.segment().is_none());
    assert_eq!(ring.monitor(), Some(&chosen));
    let written: Vec<_> = fs::read_dir(&folder.path).unwrap().collect();
    assert!(
        written.is_empty(),
        "another monitor must not put a file in the otkat folder"
    );
}

#[test]
fn before_the_screen_answer_a_monitor_does_not_enter_the_ring() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::enter_session(folder.path.clone(), None);

    ring.take_monitor(MonitorMoment::new(
        Screen::new("Alexey's screen"),
        Duration::from_secs(2),
    ));

    assert!(ring.segment().is_none());
    assert!(ring.monitor().is_none());
    assert_eq!(
        ring.bind(Ring::default_bind()),
        Some(PlashkaEvent::NoAccess)
    );
    let written: Vec<_> = fs::read_dir(&folder.path).unwrap().collect();
    assert!(
        written.is_empty(),
        "a monitor before the screen answer must not put a file in the otkat folder"
    );
}
