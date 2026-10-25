use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use otkati::{BindDay, Encoder, MonitorMoment, PlashkaEvent, Ring, RingPlace, Screen};

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

    let segment = ring.segment().expect("the chosen monitor entered the ring");
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

#[test]
fn the_cursor_on_the_ring_monitor_is_in_the_otkat_picture() {
    let folder = OtkatFolder::new();
    let screen = Screen::new("Alexey's screen");
    let mut ring = Ring::enter_session(folder.path.clone(), None);
    ring.remember_screen(screen.clone());
    ring.take_monitor(MonitorMoment::new(screen.clone(), Duration::from_secs(2)));

    let event = ring.bind(Ring::default_bind());

    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.monitor(), Some(&screen));
    assert!(saved.cursor_in_picture());
    let path = folder.path.join(saved.name());
    assert!(
        !path.exists(),
        "the success plashka comes out before the file is in the folder"
    );
    drop(event);

    assert!(
        picture_shows_a_cursor(&path),
        "the cursor on the ring monitor is visible in the otkat picture"
    );
}

fn picture_shows_a_cursor(path: &Path) -> bool {
    let output = Command::new("ffmpeg")
        .arg("-v")
        .arg("error")
        .arg("-i")
        .arg(path)
        .arg("-frames:v")
        .arg("1")
        .arg("-f")
        .arg("rawvideo")
        .arg("-pix_fmt")
        .arg("rgb24")
        .arg("-")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ffmpeg failed to read the picture: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout.iter().any(|byte| *byte != 0)
}

#[test]
fn a_live_otkat_includes_output_sound_and_the_microphone() {
    let folder = OtkatFolder::new();
    let screen = Screen::new("Alexey's screen");
    let mut ring = Ring::enter_session(folder.path.clone(), None);
    ring.remember_screen(screen.clone());
    ring.take_monitor(MonitorMoment::new(screen, Duration::from_secs(2)));
    ring.output_sound();
    ring.microphone();

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert!(saved.output_sound());
    assert!(saved.microphone());
    let path = folder.path.join(saved.name());
    drop(event);

    assert_eq!(
        audio_stream_count(&path),
        2,
        "the otkat keeps output sound and the microphone as two streams"
    );
}

fn audio_stream_count(path: &Path) -> usize {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("a")
        .arg("-show_entries")
        .arg("stream=index")
        .arg("-of")
        .arg("csv=p=0")
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ffprobe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .count()
}

#[test]
fn the_ring_keeps_an_encoded_segment_not_raw_frames() {
    let folder = OtkatFolder::new();
    let screen = Screen::new("Alexey's screen");
    let mut ring = Ring::enter_session(folder.path.clone(), Some(screen.clone()));
    ring.take_monitor(MonitorMoment::new(screen, Duration::from_secs(2)).unique_frames(120));

    let segment = ring.segment().expect("the ring is holding its monitor");
    assert!(segment.is_encoded());

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert!(saved.is_encoded());
    let path = folder.path.join(saved.name());
    drop(event);

    assert_eq!(video_codec(&path), "h264");
}

fn video_codec(path: &Path) -> String {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("v:0")
        .arg("-show_entries")
        .arg("stream=codec_name")
        .arg("-of")
        .arg("default=nw=1:nk=1")
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ffprobe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

#[test]
fn the_ring_aims_for_sixty_frames_a_second_without_sixty_unique_frames() {
    let folder = OtkatFolder::new();
    let screen = Screen::new("Alexey's screen");
    let mut ring = Ring::enter_session(folder.path.clone(), Some(screen.clone()));
    ring.take_monitor(MonitorMoment::new(screen, Duration::from_secs(1)).unique_frames(7));

    assert_eq!(ring.picture_rate(), 60);
    let segment = ring.segment().expect("the monitor entered the ring");
    assert_eq!(segment.frames_per_second(), 60);

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.frames_per_second(), 60);
    let path = folder.path.join(saved.name());
    drop(event);

    assert_eq!(video_frame_rate(&path), 60);
}

fn video_frame_rate(path: &Path) -> u32 {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("v:0")
        .arg("-show_entries")
        .arg("stream=r_frame_rate")
        .arg("-of")
        .arg("default=nw=1:nk=1")
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ffprobe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    let (num, den) = text.trim().split_once('/').unwrap();
    let num: u32 = num.parse().unwrap();
    let den: u32 = den.parse().unwrap();
    assert_eq!(den, 1, "frame rate {text}");
    num
}

#[test]
fn until_the_knobs_change_the_ring_uses_memory_h264_full_sides_and_25_mbit() {
    let folder = OtkatFolder::new();
    let screen = Screen::new("Alexey's screen");
    let mut ring = Ring::enter_session(folder.path.clone(), Some(screen.clone()));

    assert_eq!(ring.place(), RingPlace::Memory);
    assert_eq!(ring.encoder(), Encoder::H264);
    assert_eq!(ring.side_share(), 100);
    assert_eq!(ring.bitrate_mbps(), 25);

    ring.take_monitor(MonitorMoment::new(screen, Duration::from_secs(2)));
    assert_eq!(ring.place(), RingPlace::Memory);
    let written: Vec<_> = fs::read_dir(&folder.path).unwrap().collect();
    assert!(
        written.is_empty(),
        "the encoded ring stays in memory, not in the otkat folder"
    );
}

#[test]
fn a_moment_without_a_cursor_or_sound_is_saved_without_them() {
    let folder = OtkatFolder::new();
    let screen = Screen::new("Alexey's screen");
    let mut ring = Ring::enter_session(folder.path.clone(), None);
    ring.remember_screen(screen.clone());
    ring.take_monitor(
        MonitorMoment::new(screen, Duration::from_secs(1))
            .cursor(false)
            .picture(0, 180, 0),
    );

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert!(!saved.cursor_in_picture());
    assert!(!saved.output_sound());
    assert!(!saved.microphone());
    let path = folder.path.join(saved.name());
    drop(event);

    assert_eq!(audio_stream_count(&path), 0);
    let pixel = pixel_at(&path, 0.0);
    assert!(
        pixel.1 > pixel.0 && pixel.1 > pixel.2 && pixel.1 > 100,
        "the picture is the moment the ring was given, got {pixel:?}"
    );
}

#[test]
fn the_otkat_keeps_the_sound_the_moment_actually_had() {
    let folder = OtkatFolder::new();
    let screen = Screen::new("Alexey's screen");
    let mut ring = Ring::enter_session(folder.path.clone(), None);
    ring.remember_screen(screen.clone());
    ring.take_monitor(
        MonitorMoment::new(screen, Duration::from_secs(1))
            .cursor(false)
            .picture(0, 180, 0)
            .output_sound_hz(330)
            .microphone_hz(990),
    );

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert!(!saved.cursor_in_picture());
    assert!(saved.output_sound());
    assert!(saved.microphone());
    let path = folder.path.join(saved.name());
    drop(event);

    let output = tone_level(&path, 0, 330);
    let stand_in = tone_level(&path, 0, 440);
    assert!(
        output > stand_in + 6.0,
        "output sound is the moment's {output} dB, not a 440 Hz stand-in at {stand_in} dB"
    );
    let microphone = tone_level(&path, 1, 990);
    let other = tone_level(&path, 1, 880);
    assert!(
        microphone > other + 6.0,
        "the microphone is the moment's {microphone} dB, not an 880 Hz stand-in at {other} dB"
    );
}

fn pixel_at(path: &Path, at_secs: f64) -> (u8, u8, u8) {
    let output = Command::new("ffmpeg")
        .arg("-v")
        .arg("error")
        .arg("-ss")
        .arg(at_secs.to_string())
        .arg("-i")
        .arg(path)
        .arg("-frames:v")
        .arg("1")
        .arg("-f")
        .arg("rawvideo")
        .arg("-pix_fmt")
        .arg("rgb24")
        .arg("-")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ffmpeg failed to read a frame: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.len() >= 3);
    (output.stdout[0], output.stdout[1], output.stdout[2])
}

fn tone_level(path: &Path, stream: usize, hz: u32) -> f64 {
    let output = Command::new("ffmpeg")
        .arg("-v")
        .arg("info")
        .arg("-i")
        .arg(path)
        .arg("-map")
        .arg(format!("0:a:{stream}"))
        .arg("-af")
        .arg(format!(
            "bandpass=f={hz}:width_type=h:w=30,astats=metadata=1:reset=0"
        ))
        .arg("-f")
        .arg("null")
        .arg("-")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stderr);
    text.lines()
        .filter_map(|line| line.split("RMS level dB:").nth(1))
        .last()
        .and_then(|value| value.trim().parse::<f64>().ok())
        .unwrap_or(-120.0)
}

#[test]
fn a_bind_on_the_live_ring_saves_the_same_name_file_and_plashka() {
    let folder = OtkatFolder::new();
    let screen = Screen::new("Alexey's screen");
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.remember_screen(screen.clone());
    ring.take_monitor(MonitorMoment::new(screen, Duration::from_secs(120)));

    let event = ring.bind(Ring::default_bind());

    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.name(), "otkat1-2026-10-25.mp4");
    assert_eq!(saved.duration(), Duration::from_secs(120));
    let path = folder.path.join("otkat1-2026-10-25.mp4");
    assert!(
        !path.exists(),
        "the success plashka comes out before the file is in the folder"
    );
    drop(event);

    assert_eq!(mp4_duration_secs(&path), 120);
    let names: Vec<_> = fs::read_dir(&folder.path)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(
        names,
        vec![std::ffi::OsString::from("otkat1-2026-10-25.mp4")]
    );
}

fn mp4_duration_secs(path: &Path) -> u64 {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=nw=1:nk=1")
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ffprobe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    let secs: f64 = text.trim().parse().unwrap();
    secs.round() as u64
}
