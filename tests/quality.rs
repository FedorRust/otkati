use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use otkati::{BindDay, Bitrate, PlashkaEvent, ResolutionShare, Ring, Screen};

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
        let path = std::env::temp_dir().join(format!("otkati-quality-{tick}-{n}"));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for OtkatFolder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn mp4_frame(path: &Path) -> (u32, u32) {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("v:0")
        .arg("-show_entries")
        .arg("stream=width,height")
        .arg("-of")
        .arg("csv=s=x:p=0")
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ffprobe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    let (width, height) = text.trim().split_once('x').unwrap();
    (width.parse().unwrap(), height.parse().unwrap())
}

fn saved_otkat(ring: &mut Ring) -> otkati::SavedOtkat {
    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    saved
}

#[test]
fn a_new_ring_saves_the_whole_monitor_at_25_mbit() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.remember_screen(Screen::with_frame("desk", 1920, 1080));
    ring.hold(Duration::from_secs(1));

    let saved = saved_otkat(&mut ring);

    assert_eq!(saved.width(), 1920);
    assert_eq!(saved.height(), 1080);
    assert_eq!(saved.bitrate_mbit(), 25);
    let path = folder.path.join(saved.name());
    drop(saved);
    assert_eq!(mp4_frame(&path), (1920, 1080));
}

#[test]
fn the_share_applies_to_each_side_of_the_monitor() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.remember_screen(Screen::with_frame("desk", 1920, 1080));
    ring.hold(Duration::from_secs(1));

    ring.take_resolution_share(ResolutionShare::new(50).expect("50 is a resolution share"));
    let half = saved_otkat(&mut ring);
    assert_eq!(half.width(), 960);
    assert_eq!(half.height(), 540);
    let half_path = folder.path.join(half.name());
    drop(half);
    assert_eq!(mp4_frame(&half_path), (960, 540));

    ring.take_resolution_share(ResolutionShare::new(70).expect("70 is a resolution share"));
    let seventy = saved_otkat(&mut ring);
    assert_eq!(seventy.width(), 1344);
    assert_eq!(seventy.height(), 756);
    let seventy_path = folder.path.join(seventy.name());
    drop(seventy);
    assert_eq!(mp4_frame(&seventy_path), (1344, 756));
}

#[test]
fn width_and_height_come_out_even() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.hold(Duration::from_secs(1));

    ring.remember_screen(Screen::with_frame("desk", 1919, 1079));
    ring.take_resolution_share(ResolutionShare::new(100).expect("100 is a resolution share"));
    let full = saved_otkat(&mut ring);
    assert_eq!(full.width(), 1918);
    assert_eq!(full.height(), 1078);
    let full_path = folder.path.join(full.name());
    drop(full);
    assert_eq!(mp4_frame(&full_path), (1918, 1078));

    ring.remember_screen(Screen::with_frame("laptop", 1366, 768));
    ring.take_resolution_share(ResolutionShare::new(90).expect("90 is a resolution share"));
    let scaled = saved_otkat(&mut ring);
    assert_eq!(scaled.width(), 1228);
    assert_eq!(scaled.height(), 690);
    let scaled_path = folder.path.join(scaled.name());
    drop(scaled);
    assert_eq!(mp4_frame(&scaled_path), (1228, 690));
}

#[test]
fn changing_the_share_leaves_the_bitrate() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.remember_screen(Screen::with_frame("desk", 1920, 1080));
    ring.hold(Duration::from_secs(1));
    ring.take_bitrate(Bitrate::new(40));

    ring.take_resolution_share(ResolutionShare::new(50).expect("50 is a resolution share"));
    let half = saved_otkat(&mut ring);
    assert_eq!(half.width(), 960);
    assert_eq!(half.height(), 540);
    assert_eq!(half.bitrate_mbit(), 40);

    ring.take_resolution_share(ResolutionShare::new(100).expect("100 is a resolution share"));
    let full = saved_otkat(&mut ring);
    assert_eq!(full.width(), 1920);
    assert_eq!(full.height(), 1080);
    assert_eq!(full.bitrate_mbit(), 40);
}

#[test]
fn the_share_runs_from_50_to_100_in_steps_of_10() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.remember_screen(Screen::with_frame("desk", 1920, 1080));
    ring.hold(Duration::from_secs(1));

    let frames = [
        (50, 960, 540),
        (60, 1152, 648),
        (70, 1344, 756),
        (80, 1536, 864),
        (90, 1728, 972),
        (100, 1920, 1080),
    ];
    for (percent, width, height) in frames {
        let share = ResolutionShare::new(percent).expect("a ten-percent step from 50 to 100");
        ring.take_resolution_share(share);
        let saved = saved_otkat(&mut ring);
        assert_eq!(saved.width(), width, "{percent}%");
        assert_eq!(saved.height(), height, "{percent}%");
        drop(saved);
    }

    assert!(ResolutionShare::new(40).is_none());
    assert!(ResolutionShare::new(55).is_none());
    assert!(ResolutionShare::new(110).is_none());
}
