use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use otkati::{ConnectedMonitor, MonitorMoment, PlashkaEvent, Ring, Screen};

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

#[test]
fn a_connected_monitor_is_not_assigned_by_position_focus_or_connector() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::enter_session(folder.path.clone(), None);

    ring.see_connected_monitors(&[ConnectedMonitor::new(
        Screen::new("the origin"),
        (0, 0),
        true,
        "DP-1",
    )]);

    assert!(ring.asks_which_screen());
    assert!(ring.monitor().is_none());
    assert_eq!(
        ring.bind(Ring::default_bind()),
        Some(PlashkaEvent::NoAccess)
    );
    let written: Vec<_> = fs::read_dir(&folder.path).unwrap().collect();
    assert!(
        written.is_empty(),
        "position, focus, and a connector name must not put a file in the otkat folder"
    );
}

#[test]
fn another_monitor_at_the_origin_does_not_replace_the_chosen_one() {
    let folder = OtkatFolder::new();
    let desk = Screen::new("desk");
    let laptop = Screen::new("laptop");
    let mut ring = Ring::enter_session(folder.path.clone(), Some(desk.clone()));
    ring.take_monitor(MonitorMoment::new(desk.clone(), Duration::from_secs(2)));

    ring.see_connected_monitors(&[
        ConnectedMonitor::new(laptop.clone(), (0, 0), true, "DP-1"),
        ConnectedMonitor::new(desk.clone(), (1920, 0), false, "HDMI-A-1"),
    ]);

    assert_eq!(ring.monitor(), Some(&desk));
    ring.take_monitor(MonitorMoment::new(laptop, Duration::from_secs(4)));
    let segment = ring.segment().expect("the chosen monitor stays in the ring");
    assert_eq!(segment.monitor(), &desk);
    assert_eq!(segment.duration(), Duration::from_secs(2));
}

#[test]
fn two_remaining_monitors_are_not_chosen_by_position_focus_or_connector() {
    let folder = OtkatFolder::new();
    let desk = Screen::new("desk");
    let laptop = Screen::new("laptop");
    let side = Screen::new("side");
    let mut ring = Ring::enter_session(folder.path.clone(), Some(desk.clone()));
    ring.take_monitor(MonitorMoment::new(desk.clone(), Duration::from_secs(2)));

    ring.see_connected_monitors(&[
        ConnectedMonitor::new(laptop.clone(), (0, 0), true, "DP-1"),
        ConnectedMonitor::new(side.clone(), (1920, 0), false, "HDMI-A-2"),
    ]);

    assert_eq!(ring.monitor(), Some(&desk));
    ring.take_monitor(MonitorMoment::new(laptop, Duration::from_secs(4)));
    ring.take_monitor(MonitorMoment::new(side, Duration::from_secs(4)));
    let segment = ring.segment().expect("the ring does not go empty");
    assert_eq!(segment.monitor(), &desk);
    assert_eq!(segment.duration(), Duration::from_secs(2));
}

#[test]
fn a_disconnected_monitor_moves_the_ring_to_the_one_that_remains() {
    let folder = OtkatFolder::new();
    let desk = Screen::new("desk");
    let laptop = Screen::new("laptop");
    let mut ring = Ring::enter_session(folder.path.clone(), Some(desk.clone()));
    ring.take_monitor(MonitorMoment::new(desk.clone(), Duration::from_secs(4)));

    ring.see_connected_monitors(&[ConnectedMonitor::new(
        laptop.clone(),
        (0, 0),
        true,
        "DP-1",
    )]);

    assert_eq!(ring.monitor(), Some(&laptop));
    let segment = ring.segment().expect("the ring does not go empty");
    assert_eq!(segment.monitor(), &laptop);
    assert_eq!(segment.duration(), Duration::from_secs(4));

    ring.take_monitor(MonitorMoment::new(desk, Duration::from_secs(9)));
    assert_eq!(
        ring.segment().unwrap().duration(),
        Duration::from_secs(4)
    );

    ring.take_monitor(MonitorMoment::new(laptop.clone(), Duration::from_secs(3)));
    let segment = ring.segment().expect("the remaining monitor entered the ring");
    assert_eq!(segment.monitor(), &laptop);
    assert_eq!(segment.duration(), Duration::from_secs(7));

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat from the monitor that remains, got {event:?}");
    };
    assert_eq!(saved.monitor(), Some(&laptop));
    assert_eq!(saved.duration(), Duration::from_secs(7));
    let path = folder.path.join(saved.name());
    assert!(
        !path.exists(),
        "the success plashka comes out before the file is in the folder"
    );
    drop(event);
    assert!(
        path.exists(),
        "the otkat from the monitor that remains is written after the plashka"
    );
}
