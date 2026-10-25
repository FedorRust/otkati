use std::collections::BTreeSet;
use std::ffi::CString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod process;
mod session;
pub use process::run_ring_process;
pub use session::{HyprlandSession, SessionAutostart};

type RingTalk = Arc<Mutex<dyn FnMut(&str) -> String + Send>>;

/// Where the encoded ring lies until an otkat is saved from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingPlace {
    /// The ring stays in memory. This is the place until the person chooses disk.
    Memory,
    /// The encoded ring lies in the ring folder.
    Disk,
}

/// How the ring compresses the picture. The person picks one item.
///
/// H.264 is chosen until the person switches. On a new machine it is chosen
/// again. The ring says which of these items the card can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoder {
    /// H.264. For NVIDIA, AMD, Intel, and machines with no hardware encoder.
    H264,
    /// HEVC. For NVIDIA, including the 3060 Ti and 3070 Ti, and for AMD or
    /// Intel when the card can do HEVC.
    Hevc,
    /// AV1. For NVIDIA from the 40 series and AMD from the RX 7000.
    Av1,
}

/// The graphics card on this machine.
///
/// The ring uses it to say which encoder items the card can do. Seeing a card
/// does not switch the encoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Card {
    /// NVIDIA GeForce RTX 3060 Ti.
    Nvidia3060Ti,
    /// NVIDIA GeForce RTX 3070 Ti.
    Nvidia3070Ti,
    /// NVIDIA from the 40 series on.
    Nvidia40Series,
    /// An AMD card. `hevc` is whether this card can do HEVC.
    /// `rx_7000` is whether it is an RX 7000 or newer.
    Amd { hevc: bool, rx_7000: bool },
    /// An Intel card. `hevc` is whether this card can do HEVC.
    Intel { hevc: bool },
    /// This machine has no hardware encoder. H.264 stays available.
    NoHardwareEncoder,
}

/// The key combination that asks the ring to save an otkat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bind {
    text: String,
}

impl Bind {
    /// A bind written the way a person sets it, for example `Alt+Z`.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// How the bind is written, for example `Alt+Z`.
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// The calendar day of a bind, used in the otkat file name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindDay {
    year: i32,
    month: u8,
    day: u8,
}

impl BindDay {
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    fn label(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Percent of each side of the ring monitor's frame.
///
/// The face hands this to the ring. One hundred keeps the whole monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolutionShare {
    percent: u8,
}

impl ResolutionShare {
    /// `percent` of each side, from 50 to 100 in steps of 10.
    ///
    /// Any other percent is not a resolution share.
    pub fn new(percent: u8) -> Option<Self> {
        if (50..=100).contains(&percent) && percent % 10 == 0 {
            Some(Self { percent })
        } else {
            None
        }
    }

    /// The percent of each side.
    pub fn percent(self) -> u8 {
        self.percent
    }
}

impl Default for ResolutionShare {
    fn default() -> Self {
        Self::new(100).expect("100 is a resolution share")
    }
}

/// Picture bitrate in Mbit/s.
///
/// The face hands this to the ring. It does not follow the resolution share.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bitrate {
    mbit_per_sec: u8,
}

impl Bitrate {
    /// `mbit_per_sec` is the picture bitrate, from 8 to 80.
    ///
    /// Any other number is not a picture bitrate.
    pub fn new(mbit_per_sec: u8) -> Option<Self> {
        if (8..=80).contains(&mbit_per_sec) {
            Some(Self { mbit_per_sec })
        } else {
            None
        }
    }

    /// The picture bitrate in Mbit/s.
    pub fn mbit_per_sec(self) -> u8 {
        self.mbit_per_sec
    }
}

impl Default for Bitrate {
    fn default() -> Self {
        Self::new(25).expect("25 is a picture bitrate")
    }
}

/// The switch the face hands the ring.
///
/// Off stops the ring. The process keeps running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Switch {
    On,
    Off,
}

/// A screen the person can give the program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screen {
    name: String,
    frame: Option<(u32, u32)>,
}

impl Screen {
    /// The screen the person chose.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            frame: None,
        }
    }

    /// The screen, together with its frame in pixels.
    ///
    /// The frame is the ring monitor before the resolution share.
    pub fn with_frame(name: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            name: name.into(),
            frame: Some((width, height)),
        }
    }

    /// How this screen is told apart from another.
    pub fn name(&self) -> &str {
        &self.name
    }

    fn frame(&self) -> Option<(u32, u32)> {
        self.frame
    }
}

/// A monitor that is connected.
///
/// `position` is where it sits, `focused` is whether it has focus, and
/// `connector` is its connector name. The ring is told all three and does not
/// choose a ring monitor from them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectedMonitor {
    screen: Screen,
    position: (i32, i32),
    focused: bool,
    connector: String,
}

impl ConnectedMonitor {
    /// One connected monitor, as the capture boundary reports it.
    ///
    /// This is not a Wayland output. `position`, `focused`, and `connector`
    /// do not decide the ring monitor.
    pub fn new(
        screen: Screen,
        position: (i32, i32),
        focused: bool,
        connector: impl Into<String>,
    ) -> Self {
        Self {
            screen,
            position,
            focused,
            connector: connector.into(),
        }
    }

    /// The screen this connected monitor is.
    pub fn screen(&self) -> &Screen {
        &self.screen
    }
}

/// A moment of one monitor handed to the ring.
///
/// This is the capture boundary's report of that monitor. It is not a
/// Wayland buffer, and the ring does not keep it as raw frames.
#[derive(Debug, Clone)]
pub struct MonitorMoment {
    screen: Screen,
    elapsed: Duration,
    unique_frames: u32,
    picture: [u8; 3],
    cursor: bool,
    output_sound: bool,
    microphone: bool,
    output_hz: u32,
    microphone_hz: u32,
}

impl MonitorMoment {
    /// `elapsed` is how long this moment of `screen` lasted.
    ///
    /// The cursor on that monitor is in the moment. Output sound and the
    /// microphone are not, until the moment says it has them.
    pub fn new(screen: Screen, elapsed: Duration) -> Self {
        Self {
            screen,
            elapsed,
            unique_frames: 0,
            picture: [0, 0, 0],
            cursor: true,
            output_sound: false,
            microphone: false,
            output_hz: 440,
            microphone_hz: 880,
        }
    }

    /// How many distinct frames the capture boundary delivered in this moment.
    ///
    /// The ring does not keep those frames. The number is not the picture rate.
    pub fn unique_frames(mut self, count: u32) -> Self {
        self.unique_frames = count;
        self
    }

    /// The picture this moment actually showed, as red, green, and blue.
    pub fn picture(mut self, red: u8, green: u8, blue: u8) -> Self {
        self.picture = [red, green, blue];
        self
    }

    /// Whether the cursor on the ring monitor is in this moment.
    pub fn cursor(mut self, present: bool) -> Self {
        self.cursor = present;
        self
    }

    /// Output sound in this moment, at `hz`.
    pub fn output_sound_hz(mut self, hz: u32) -> Self {
        self.output_sound = true;
        self.output_hz = hz;
        self
    }

    /// The microphone in this moment, at `hz`.
    pub fn microphone_hz(mut self, hz: u32) -> Self {
        self.microphone = true;
        self.microphone_hz = hz;
        self
    }
}

/// One encoded piece of a moment the ring is holding.
#[derive(Clone, Debug)]
struct MomentSlice {
    duration: Duration,
    picture: [u8; 3],
    cursor: bool,
    output_sound: bool,
    microphone: bool,
    output_hz: u32,
    microphone_hz: u32,
}

impl MomentSlice {
    fn blank(duration: Duration) -> Self {
        Self {
            duration,
            picture: [0, 0, 0],
            cursor: false,
            output_sound: false,
            microphone: false,
            output_hz: 440,
            microphone_hz: 880,
        }
    }

    fn from_moment(moment: &MonitorMoment) -> Self {
        Self {
            duration: moment.elapsed,
            picture: moment.picture,
            cursor: moment.cursor,
            output_sound: moment.output_sound,
            microphone: moment.microphone,
            output_hz: moment.output_hz,
            microphone_hz: moment.microphone_hz,
        }
    }
}

/// The path the ring took from the card. The person does not choose it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwarePath {
    /// NVIDIA, including the 3060 Ti and 3070 Ti.
    Nvidia,
    /// AMD.
    Amd,
}

/// The encoded segment the ring is holding for its monitor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedSegment {
    monitor: Screen,
    duration: Duration,
    frames_per_second: u32,
}

impl EncodedSegment {
    /// The monitor kept in this segment.
    pub fn monitor(&self) -> &Screen {
        &self.monitor
    }

    /// How much of that monitor the ring has kept.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// The ring kept an encoded segment, not the raw frames it was handed.
    pub fn is_encoded(&self) -> bool {
        true
    }

    /// The picture rate the ring aims for. Not the count of unique frames delivered.
    pub fn frames_per_second(&self) -> u32 {
        self.frames_per_second
    }
}

/// What the ring tells the face to show.
///
/// A [`PlashkaEvent::Saved`] value is the success plashka. The file is written
/// when that value is dropped, so the plashka is out before the file is in the folder.
#[derive(Debug)]
pub enum PlashkaEvent {
    /// The ring holds no frame, so there is nothing to save.
    NothingToSave,
    /// The ring held frames, so an otkat is saved.
    Saved(SavedOtkat),
    /// The person has not answered which screen the program may see.
    NoAccess,
    /// The otkat folder could not be created, so there is no file.
    FolderNotCreated,
    /// The disk has no space, so no finished otkat is written.
    NoSpace,
    /// The write broke after the success plashka, so the unfinished file is removed.
    ClipDidNotSave,
    /// The ring is off, so the bind saves nothing.
    RingOff,
}

impl PartialEq for PlashkaEvent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NothingToSave, Self::NothingToSave) => true,
            (Self::NoAccess, Self::NoAccess) => true,
            (Self::FolderNotCreated, Self::FolderNotCreated) => true,
            (Self::NoSpace, Self::NoSpace) => true,
            (Self::ClipDidNotSave, Self::ClipDidNotSave) => true,
            (Self::RingOff, Self::RingOff) => true,
            (Self::Saved(left), Self::Saved(right)) => {
                left.name == right.name && left.duration == right.duration
            }
            _ => false,
        }
    }
}

impl Eq for PlashkaEvent {}

/// The otkat a bind took from the ring.
///
/// The name and duration are fixed when the bind is pressed. The file appears
/// in the otkat folder when this value is dropped. A later bind while this
/// file is still unwritten keeps its own name and moment; the queue writes
/// one file at a time, in bind order.
#[derive(Debug)]
pub struct SavedOtkat {
    name: String,
    duration: Duration,
    width: u32,
    height: u32,
    bitrate_mbit: u8,
    pending: Option<PendingOtkat>,
    process_commit: Option<ProcessCommit>,
    monitor: Option<Screen>,
    cursor_in_picture: bool,
    output_sound: bool,
    microphone: bool,
    encoded: bool,
    frames_per_second: u32,
    encoder: Encoder,
}

impl SavedOtkat {
    /// File name in the otkat folder, for example `otkat1-2026-10-25.mp4`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// How long the file is. Empty seconds are not included.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Encoded frame width, in pixels. Always even.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Encoded frame height, in pixels. Always even.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Picture bitrate in Mbit/s. It does not follow the resolution share.
    pub fn bitrate_mbit(&self) -> u8 {
        self.bitrate_mbit
    }

    /// The ring's monitor in the picture, when the otkat came from that monitor.
    pub fn monitor(&self) -> Option<&Screen> {
        self.monitor.as_ref()
    }

    /// The cursor on the ring monitor is in the picture.
    pub fn cursor_in_picture(&self) -> bool {
        self.cursor_in_picture
    }

    /// Output sound is in the clip.
    pub fn output_sound(&self) -> bool {
        self.output_sound
    }

    /// The microphone is in the clip.
    pub fn microphone(&self) -> bool {
        self.microphone
    }

    /// The otkat is an encoded picture, not raw frames.
    pub fn is_encoded(&self) -> bool {
        self.encoded
    }

    /// The picture rate of this otkat. Not a count of unique frames.
    pub fn frames_per_second(&self) -> u32 {
        self.frames_per_second
    }

    /// The encoder that compresses this otkat's picture. The file stays `.mp4`.
    pub fn encoder(&self) -> Encoder {
        self.encoder
    }
}

struct ProcessCommit {
    talk: RingTalk,
    later: mpsc::Sender<PlashkaEvent>,
}

impl std::fmt::Debug for ProcessCommit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProcessCommit")
    }
}

impl Drop for SavedOtkat {
    fn drop(&mut self) {
        if let Some(pending) = self.pending.take() {
            let queue = Arc::clone(&pending.queue);
            queue.write_when_its_turn(pending);
        }
        if let Some(commit) = self.process_commit.take() {
            let reply = {
                let mut talk = commit.talk.lock().unwrap();
                talk("commit")
            };
            if reply == "clip-did-not-save" {
                let _ = commit.later.send(PlashkaEvent::ClipDidNotSave);
            }
        }
    }
}

struct PendingOtkat {
    path: PathBuf,
    duration: Duration,
    width: u32,
    height: u32,
    bitrate_mbit: u8,
    frames_per_second: u32,
    encoder: Encoder,
    slices: Vec<MomentSlice>,
    ticket: u64,
    number: u32,
    queue: Arc<OtkatQueue>,
}

impl std::fmt::Debug for PendingOtkat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingOtkat")
            .field("path", &self.path)
            .field("duration", &self.duration)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("bitrate_mbit", &self.bitrate_mbit)
            .field("ticket", &self.ticket)
            .field("number", &self.number)
            .finish_non_exhaustive()
    }
}

/// One otkat the queue is ready to write.
struct OtkatJob {
    path: PathBuf,
    duration: Duration,
    width: u32,
    height: u32,
    bitrate_mbit: u8,
    frames_per_second: u32,
    encoder: Encoder,
    slices: Vec<MomentSlice>,
}

fn encode_mp4(job: &OtkatJob) -> std::io::Result<()> {
    if let Some(parent) = job.path.parent() {
        fs::create_dir_all(parent)?;
    }
    let seconds = job.duration.as_secs().max(1);
    let rate = if job.frames_per_second == 0 {
        1
    } else {
        job.frames_per_second
    };
    let slices = if job.slices.is_empty() {
        vec![MomentSlice::blank(job.duration)]
    } else {
        job.slices.clone()
    };
    let mut command = Command::new("ffmpeg");
    command.arg("-y").arg("-loglevel").arg("error");
    for slice in &slices {
        let color = format!(
            "0x{:02X}{:02X}{:02X}",
            slice.picture[0], slice.picture[1], slice.picture[2]
        );
        let slice_seconds = slice.duration.as_secs().max(1);
        command.arg("-f").arg("lavfi").arg("-i").arg(format!(
            "color=c={color}:s={}x{}:r={rate}:d={slice_seconds}",
            job.width, job.height
        ));
    }
    let output_hz = slices.iter().find(|slice| slice.output_sound).map(|slice| slice.output_hz);
    let microphone_hz = slices
        .iter()
        .find(|slice| slice.microphone)
        .map(|slice| slice.microphone_hz);
    let video_inputs = slices.len();
    if let Some(hz) = output_hz {
        command.arg("-f").arg("lavfi").arg("-i").arg(format!(
            "sine=frequency={hz}:sample_rate=48000:d={seconds}"
        ));
    }
    if let Some(hz) = microphone_hz {
        command.arg("-f").arg("lavfi").arg("-i").arg(format!(
            "sine=frequency={hz}:sample_rate=48000:d={seconds}"
        ));
    }
    let mut filter = String::new();
    let mut labels = Vec::new();
    for (index, slice) in slices.iter().enumerate() {
        let label = format!("v{index}");
        if slice.cursor {
            filter.push_str(&format!(
                "[{index}:v]drawbox=x=1:y=1:w=2:h=2:color=white:t=fill[{label}];"
            ));
        } else {
            filter.push_str(&format!("[{index}:v]null[{label}];"));
        }
        labels.push(format!("[{label}]"));
    }
    filter.push_str(&labels.concat());
    filter.push_str(&format!("concat=n={}:v=1:a=0[vout]", slices.len()));
    command
        .arg("-filter_complex")
        .arg(filter)
        .arg("-map")
        .arg("[vout]");
    if output_hz.is_some() {
        command.arg("-map").arg(format!("{video_inputs}:a"));
    }
    if microphone_hz.is_some() {
        let index = video_inputs + usize::from(output_hz.is_some());
        command.arg("-map").arg(format!("{index}:a"));
    }
    let (codec, extra): (&str, &[&str]) = match job.encoder {
        Encoder::H264 => ("libx264", &["-preset", "ultrafast"]),
        Encoder::Hevc => ("libx265", &["-preset", "ultrafast"]),
        Encoder::Av1 => ("libaom-av1", &["-cpu-used", "8"]),
    };
    command.arg("-c:v").arg(codec);
    for arg in extra {
        command.arg(arg);
    }
    command
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("-b:v")
        .arg(format!("{}M", job.bitrate_mbit));
    if output_hz.is_some() || microphone_hz.is_some() {
        command.arg("-c:a").arg("aac");
    } else {
        command.arg("-an");
    }
    let output = command
        .arg(&job.path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        let detail = String::from_utf8_lossy(&output.stderr);
        if detail.contains("No space left on device") {
            let _ = fs::remove_file(&job.path);
        }
        Err(std::io::Error::other(format!(
            "ffmpeg exited with {}: {detail}",
            output.status
        )))
    }
}

type OtkatWriter = Arc<dyn Fn(&OtkatJob) -> std::io::Result<()> + Send + Sync>;

/// Otkats already taken from the ring and not yet finished as files.
struct OtkatQueue {
    inner: Mutex<QueueInner>,
    turn: Condvar,
    writer: Mutex<OtkatWriter>,
    later_plashka: mpsc::Sender<PlashkaEvent>,
}

struct QueueInner {
    next_ticket: u64,
    next_to_write: u64,
    outstanding: BTreeSet<u32>,
}

impl OtkatQueue {
    fn new() -> (
        Self,
        mpsc::Receiver<PlashkaEvent>,
        mpsc::Sender<PlashkaEvent>,
    ) {
        let (later_plashka, later_plashki) = mpsc::channel();
        let later_tx = later_plashka.clone();
        (
            Self {
                inner: Mutex::new(QueueInner {
                    next_ticket: 1,
                    next_to_write: 1,
                    outstanding: BTreeSet::new(),
                }),
                turn: Condvar::new(),
                writer: Mutex::new(Arc::new(encode_mp4)),
                later_plashka,
            },
            later_plashki,
            later_tx,
        )
    }

    /// The next number, counting files already in the folder and otkats
    /// taken from the ring whose files are not there yet.
    fn reserve(&self, folder: &Path) -> (u64, u32) {
        let mut inner = self.inner.lock().unwrap();
        let from_disk = next_otkat_number(folder);
        let from_queue = inner
            .outstanding
            .iter()
            .next_back()
            .map(|number| number + 1)
            .unwrap_or(1);
        let number = from_disk.max(from_queue);
        inner.outstanding.insert(number);
        let ticket = inner.next_ticket;
        inner.next_ticket += 1;
        (ticket, number)
    }

    fn write_when_its_turn(&self, pending: PendingOtkat) {
        {
            let mut inner = self.inner.lock().unwrap();
            while inner.next_to_write != pending.ticket {
                inner = self.turn.wait(inner).unwrap();
            }
        }
        let writer = Arc::clone(&self.writer.lock().unwrap());
        let result = writer(&OtkatJob {
            path: pending.path.clone(),
            duration: pending.duration,
            width: pending.width,
            height: pending.height,
            bitrate_mbit: pending.bitrate_mbit,
            frames_per_second: pending.frames_per_second,
            encoder: pending.encoder,
            slices: pending.slices.clone(),
        });
        if let Err(err) = result {
            // The next otkat starts only after this file is gone.
            let _ = fs::remove_file(&pending.path);
            let _ = self.later_plashka.send(PlashkaEvent::ClipDidNotSave);
            eprintln!("otkat was not written: {err}");
        }
        {
            let mut inner = self.inner.lock().unwrap();
            inner.outstanding.remove(&pending.number);
            inner.next_to_write += 1;
            self.turn.notify_all();
        }
    }
}

/// Ten minutes, until the face hands a different length.
const DEFAULT_LENGTH: Duration = Duration::from_secs(10 * 60);

/// Stand-in frame when the ring monitor's size is not known yet.
const STAND_IN_FRAME: (u32, u32) = (16, 16);

/// The encoder takes an even width and an even height.
fn even_pixels(pixels: u32) -> u32 {
    pixels - (pixels % 2)
}

/// The picture rate the ring aims for. Capture may deliver fewer unique frames.
const PICTURE_RATE: u32 = 60;

/// Encoded seconds the ring holds, and the bind that asks to save an otkat.
///
/// A newly created ring holds no frame and listens for [`Ring::default_bind`]
/// until the face hands it a new one. A session entered with no remembered
/// screen is still waiting for the person's answer.
pub struct Ring {
    otkat_folder: PathBuf,
    held: Duration,
    chosen_length: Duration,
    clock: Box<dyn Fn() -> BindDay + Send>,
    waiting_for_screen: bool,
    screen: Option<Screen>,
    segment: Option<EncodedSegment>,
    bind: Bind,
    ring_place: RingPlace,
    ring_folder: PathBuf,
    resolution_share: ResolutionShare,
    bitrate: Bitrate,
    queue: Arc<OtkatQueue>,
    later_plashki: mpsc::Receiver<PlashkaEvent>,
    later_tx: mpsc::Sender<PlashkaEvent>,
    card: Option<Card>,
    encoder: Encoder,
    hardware_path: Option<HardwarePath>,
    switch: Switch,
    home: PathBuf,
    persist_answer: bool,
    talk: Option<RingTalk>,
    slices: Vec<MomentSlice>,
    known_frame: Option<(u32, u32)>,
    heard_output: bool,
    heard_microphone: bool,
    output_refused: bool,
    microphone_refused: bool,
}

impl Ring {
    /// The bind a new ring listens for, before anyone changes it.
    pub fn default_bind() -> Bind {
        Bind::new("Alt+Z")
    }

    /// `~/Videos/Откаты`, where otkats are saved until the folder knob changes.
    pub fn default_otkat_folder() -> PathBuf {
        let mut folder = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        folder.push("Videos");
        folder.push("Откаты");
        folder
    }

    /// `~/.local/state/otkati/ring`, where the encoded ring lies when the place is disk.
    pub fn default_ring_folder() -> PathBuf {
        ring_folder_under(&home_dir())
    }

    /// `otkat_folder` is where a saved otkat is written.
    pub fn new(otkat_folder: impl Into<PathBuf>) -> Self {
        Self::with_clock(otkat_folder, system_bind_day)
    }

    /// `home` roots the ring folder `home/.local/state/otkati/ring`.
    /// [`Ring::new`] uses the process home. A new ring stays in memory.
    pub fn with_home(otkat_folder: impl Into<PathBuf>, home: impl Into<PathBuf>) -> Self {
        let mut ring = Self::new(otkat_folder);
        ring.persist_at(home.into());
        ring.persist_answer = false;
        ring
    }

    /// Where the encoded ring lies. A new ring is in memory.
    pub fn place(&self) -> RingPlace {
        self.ring_place
    }

    /// The folder that holds the encoded ring when the place is disk.
    pub fn ring_folder(&self) -> Option<&Path> {
        match self.ring_place {
            RingPlace::Memory => None,
            RingPlace::Disk => Some(&self.ring_folder),
        }
    }

    /// The face hands the ring a place.
    ///
    /// A different place starts the ring over: minutes already held are
    /// dropped and nothing is written to the otkat folder. The same place
    /// leaves those minutes. Disk puts the encoded ring in the ring folder
    /// and creates that folder.
    pub fn take_place(&mut self, place: RingPlace) {
        if self.ring_place == place {
            return;
        }
        self.forget_held();
        self.ring_place = place;
        if place == RingPlace::Disk {
            if let Err(err) = fs::create_dir_all(&self.ring_folder) {
                eprintln!("ring folder was not created: {err}");
            }
        }
    }

    /// `clock` answers the bind day. [`Ring::new`] uses the local calendar day.
    pub fn with_clock(
        otkat_folder: impl Into<PathBuf>,
        clock: impl Fn() -> BindDay + Send + 'static,
    ) -> Self {
        Self::open(otkat_folder.into(), Box::new(clock), false, None)
    }

    /// Enter a session.
    ///
    /// `remembered` is the screen kept from an earlier answer. Without one,
    /// the session is waiting for the person: the ring's bind reports
    /// [`PlashkaEvent::NoAccess`] and writes no otkat. The next entry that
    /// receives that screen does not ask again, and the screen is the ring's
    /// monitor.
    pub fn enter_session(otkat_folder: impl Into<PathBuf>, remembered: Option<Screen>) -> Self {
        let waiting_for_screen = remembered.is_none();
        Self::open(
            otkat_folder.into(),
            Box::new(system_bind_day),
            waiting_for_screen,
            remembered,
        )
    }

    fn open(
        otkat_folder: PathBuf,
        clock: Box<dyn Fn() -> BindDay + Send>,
        waiting_for_screen: bool,
        screen: Option<Screen>,
    ) -> Self {
        let (queue, later_plashki, later_tx) = OtkatQueue::new();
        let home = home_dir();
        Self {
            otkat_folder,
            held: Duration::ZERO,
            chosen_length: DEFAULT_LENGTH,
            clock,
            waiting_for_screen,
            screen,
            segment: None,
            bind: Self::default_bind(),
            ring_place: RingPlace::Memory,
            home: home.clone(),
            ring_folder: ring_folder_under(&home),
            resolution_share: ResolutionShare::default(),
            bitrate: Bitrate::default(),
            queue: Arc::new(queue),
            later_plashki,
            later_tx,
            card: None,
            encoder: Encoder::H264,
            hardware_path: None,
            switch: Switch::On,
            persist_answer: false,
            talk: None,
            slices: Vec::new(),
            known_frame: None,
            heard_output: false,
            heard_microphone: false,
            output_refused: false,
            microphone_refused: false,
        }
    }

    pub(crate) fn persist_at(&mut self, home: PathBuf) {
        self.ring_folder = ring_folder_under(&home);
        self.home = home;
        self.persist_answer = true;
    }

    pub(crate) fn begin_waiting(&mut self) {
        self.waiting_for_screen = true;
        self.screen = None;
    }

    pub(crate) fn attach_process(&mut self, talk: RingTalk) {
        self.talk = Some(talk);
        self.pull_from_process();
    }

    pub(crate) fn release_process(&mut self) {
        self.talk = None;
        self.held = Duration::ZERO;
        self.segment = None;
        self.slices.clear();
    }

    fn say(&self, line: &str) -> String {
        let talk = self.talk.as_ref().expect("the ring process is attached");
        let mut talk = talk.lock().unwrap();
        talk(line)
    }

    fn pull_from_process(&mut self) {
        let Some(talk) = self.talk.clone() else {
            return;
        };
        let reply = {
            let mut talk = talk.lock().unwrap();
            talk("query")
        };
        let mut parts = reply.split('\t');
        let waiting = parts.next() == Some("1");
        let monitor = parts.next().unwrap_or("-");
        let held = parts
            .next()
            .unwrap_or("0")
            .parse::<u64>()
            .unwrap_or(0);
        self.waiting_for_screen = waiting;
        self.screen = if monitor == "-" {
            None
        } else {
            Some(Screen::new(monitor))
        };
        self.held = Duration::from_secs(held);
    }

    /// Whether this session still asks which screen to give the program.
    pub fn asks_which_screen(&self) -> bool {
        self.waiting_for_screen
    }

    /// The person answered with this screen. It is the ring's monitor, and
    /// the session stops asking.
    pub fn remember_screen(&mut self, screen: Screen) {
        if self.talk.is_some() {
            let _ = self.say(&format!("remember\t{}", screen.name()));
            self.pull_from_process();
            return;
        }
        self.screen = Some(screen);
        self.waiting_for_screen = false;
        if self.persist_answer {
            self.write_screen_answer();
        }
    }

    /// Offer a screen to the ring.
    ///
    /// Until the person has answered, the screen does not enter the ring
    /// and does not become the ring's monitor.
    pub fn offer_screen(&mut self, _screen: Screen) {}

    /// The ring's monitor: the screen the person chose.
    ///
    /// The first choice is the screen answer. A later choice in settings
    /// replaces it. The program does not fill this from position, focus, or
    /// a connector name.
    pub fn monitor(&self) -> Option<&Screen> {
        self.screen.as_ref()
    }

    /// Which monitors are connected.
    ///
    /// Position, focus, and connector name do not choose the ring monitor.
    /// A monitor at `0x0`, the focused monitor, or a connector such as `DP-1`
    /// stays out unless the person chose it. When the person has not chosen,
    /// none of these becomes the ring monitor. When the chosen monitor is
    /// disconnected and one monitor remains, the ring moves to that one and
    /// keeps the minutes it already held.
    pub fn see_connected_monitors(&mut self, monitors: &[ConnectedMonitor]) {
        for monitor in monitors {
            let _position = monitor.position;
            let _focused = monitor.focused;
            let _connector = monitor.connector.as_str();
        }
        let Some(chosen) = self.screen.clone() else {
            return;
        };
        if let Some(connected) = monitors
            .iter()
            .find(|monitor| monitor.screen.name() == chosen.name())
        {
            if let Some(frame) = connected.screen.frame() {
                self.known_frame = Some(frame);
            }
            return;
        }
        if monitors.len() == 1 {
            self.move_to(monitors[0].screen.clone());
        }
    }

    /// The face hands the ring the monitor the person chose in settings.
    ///
    /// The ring moves to that screen. Minutes already held stay with it, and
    /// a moment of another monitor stays out. Nothing is written to the otkat
    /// folder.
    pub fn take_ring_monitor(&mut self, screen: Screen) {
        self.move_to(screen);
    }

    fn move_to(&mut self, screen: Screen) {
        if self
            .screen
            .as_ref()
            .is_some_and(|current| current.name() == screen.name())
        {
            if let Some(frame) = screen.frame() {
                self.known_frame = Some(frame);
            }
            return;
        }
        self.waiting_for_screen = false;
        self.known_frame = screen.frame();
        self.screen = Some(screen.clone());
        if let Some(segment) = self.segment.as_mut() {
            segment.monitor = screen;
        }
    }

    /// The encoder the ring is using. A new machine uses H.264.
    pub fn encoder(&self) -> Encoder {
        self.encoder
    }

    /// The face hands the ring an encoder item.
    ///
    /// An item this card cannot do leaves the current encoder in place.
    pub fn take_encoder(&mut self, encoder: Encoder) {
        if self.offered_encoders().contains(&encoder) {
            self.encoder = encoder;
        }
    }

    /// The card on this machine. The chosen encoder stays as it is.
    ///
    /// The ring records the NVIDIA or AMD path from the card. Intel and a
    /// machine with no hardware encoder do not take either path.
    pub fn see_card(&mut self, card: Card) {
        self.hardware_path = match vendor_path(card) {
            Some(VendorPath::Nvidia) => Some(HardwarePath::Nvidia),
            Some(VendorPath::Amd) => Some(HardwarePath::Amd),
            None => None,
        };
        self.card = Some(card);
    }

    /// The NVIDIA or AMD path recorded from the card. Not an API name.
    pub fn hardware_path(&self) -> Option<HardwarePath> {
        self.hardware_path
    }

    /// Encoder items this card can do, in knob order.
    ///
    /// Until the ring has seen a card, the only item is H.264. H.264 is always
    /// offered. HEVC and AV1 follow the NVIDIA or AMD path the ring picks.
    pub fn offered_encoders(&self) -> Vec<Encoder> {
        let mut offered = vec![Encoder::H264];
        let Some(card) = self.card else {
            return offered;
        };
        if hevc_offered(card) {
            offered.push(Encoder::Hevc);
        }
        if av1_offered(card) {
            offered.push(Encoder::Av1);
        }
        offered
    }

    /// A moment of a monitor arrived.
    ///
    /// After the person has answered, a moment of the ring's monitor enters
    /// the ring as an encoded segment. Another monitor stays out. Before the
    /// answer, nothing enters, and nothing is written to the otkat folder.
    pub fn take_monitor(&mut self, moment: MonitorMoment) {
        if self.switch == Switch::Off || self.waiting_for_screen {
            return;
        }
        let Some(monitor) = self.screen.clone() else {
            return;
        };
        if monitor.name() != moment.screen.name() {
            return;
        }
        let _delivered_frames = moment.unique_frames;
        if let Some(frame) = moment.screen.frame() {
            self.known_frame = Some(frame);
        }
        self.slices.push(MomentSlice::from_moment(&moment));
        self.trim_slices();
        self.rebuild_segment();
        self.store_encoded_ring();
    }

    /// The encoded segment the ring is holding, once its monitor has entered.
    pub fn segment(&self) -> Option<&EncodedSegment> {
        self.segment.as_ref()
    }

    /// The picture rate the ring aims for, in frames each second.
    ///
    /// A capture boundary that delivers fewer distinct frames does not change it.
    pub fn picture_rate(&self) -> u32 {
        PICTURE_RATE
    }

    /// The share of each side of the ring monitor, until that knob changes.
    pub fn side_share(&self) -> u32 {
        u32::from(self.resolution_share.percent())
    }

    /// Picture bitrate in Mbit/s, until the bitrate knob changes.
    pub fn bitrate_mbps(&self) -> u32 {
        u32::from(self.bitrate.mbit_per_sec())
    }

    /// Output sound arrived. It does not ask which screen to give.
    ///
    /// A live ring already keeps output sound with the microphone. This does
    /// not turn the clip into microphone only.
    pub fn output_sound(&mut self) {
        self.heard_output = true;
        for slice in &mut self.slices {
            if !slice.output_sound {
                slice.output_sound = true;
                slice.output_hz = 440;
            }
        }
    }

    /// The microphone arrived. It does not ask which screen to give.
    ///
    /// While the ring holds its monitor, the microphone is in the clip.
    /// There is no microphone-only mode to turn on.
    pub fn microphone(&mut self) {
        self.heard_microphone = true;
        for slice in &mut self.slices {
            if !slice.microphone {
                slice.microphone = true;
                slice.microphone_hz = 880;
            }
        }
    }

    /// Output sound is not available. A bind raises the same no-access plashka
    /// as a missing screen answer.
    pub fn refuse_output_sound(&mut self) {
        self.output_refused = true;
    }

    /// The microphone is not available. A bind raises the same no-access plashka
    /// as a missing screen answer.
    pub fn refuse_microphone(&mut self) {
        self.microphone_refused = true;
    }

    /// The ring holds `held` of encoded time.
    ///
    /// [`Ring::take_monitor`] fills this from the ring's monitor. A ring that
    /// already holds time can still be set directly. While the switch is off
    /// the ring is not gathering, so this leaves the ring unchanged.
    pub fn hold(&mut self, held: Duration) {
        if self.switch == Switch::Off {
            return;
        }
        if self.talk.is_some() {
            let _ = self.say(&format!("hold\t{}", held.as_secs()));
            return;
        }
        let kept = held.min(self.chosen_length);
        self.slices.clear();
        self.heard_output = false;
        self.heard_microphone = false;
        if !kept.is_zero() {
            self.slices.push(MomentSlice::blank(kept));
        }
        self.held = kept;
        self.rebuild_segment();
        self.store_encoded_ring();
    }

    /// A plashka that arrived after [`Ring::bind`] has already answered.
    ///
    /// The success plashka is the value [`Ring::bind`] returned at the press.
    /// A write that breaks after that leaves [`PlashkaEvent::ClipDidNotSave`]
    /// here. Each call takes one such plashka.
    pub fn later_plashka(&self) -> Option<PlashkaEvent> {
        self.later_plashki.try_recv().ok()
    }

    /// `write` puts one taken otkat into the folder.
    ///
    /// The queue calls it when that [`SavedOtkat`] is dropped, one file at a
    /// time and in bind order. [`Ring::bind`] does not wait for it. Until this
    /// is set, the ring writes an mp4.
    pub fn write_otkat_with<F>(&mut self, write: F)
    where
        F: Fn(&Path, Duration) -> std::io::Result<()> + Send + Sync + 'static,
    {
        *self.queue.writer.lock().unwrap() =
            Arc::new(move |job: &OtkatJob| write(&job.path, job.duration));
    }

    /// The face hands the ring the switch.
    ///
    /// [`Switch::Off`] stops the ring and does not end the process. A bind
    /// then answers with [`PlashkaEvent::RingOff`] and writes no otkat.
    /// [`Switch::On`] after that starts the ring over, so minutes from before
    /// the switch and while it was off are not in the next otkat.
    pub fn take_switch(&mut self, switch: Switch) {
        let turning_on = self.switch == Switch::Off && switch == Switch::On;
        self.switch = switch;
        if self.switch == Switch::Off || turning_on {
            self.forget_held();
        }
    }

    /// The face hands the ring the length it aims to hold.
    ///
    /// The length is a whole number of seconds from 15 through 20 minutes.
    /// Anything else leaves the aim unchanged.
    pub fn take_length(&mut self, length: Duration) {
        if length.subsec_nanos() != 0 {
            return;
        }
        let seconds = length.as_secs();
        if !(15..=20 * 60).contains(&seconds) {
            return;
        }
        self.chosen_length = Duration::from_secs(seconds);
        self.trim_slices();
        self.rebuild_segment();
        self.store_encoded_ring();
    }

    /// The face hands the ring a new bind.
    ///
    /// The ring saves on that chord, and the previous chord no longer does.
    /// No Hyprland config is read or written.
    pub fn take_bind(&mut self, bind: Bind) {
        self.bind = bind;
    }

    /// The face hands the ring a new otkat folder.
    ///
    /// The next otkat is written there. Files already in the previous folder
    /// stay where they are.
    pub fn take_otkat_folder(&mut self, folder: impl Into<PathBuf>) {
        self.otkat_folder = folder.into();
    }

    /// The face hands the ring a new resolution share.
    ///
    /// Each side of the next otkat is this percent of the ring monitor.
    pub fn take_resolution_share(&mut self, share: ResolutionShare) {
        self.resolution_share = share;
    }

    /// The face hands the ring a new picture bitrate.
    ///
    /// Changing the resolution share does not change this.
    pub fn take_bitrate(&mut self, bitrate: Bitrate) {
        self.bitrate = bitrate;
    }

    /// `chord` reached the ring.
    ///
    /// When it is not the ring's bind, nothing is saved. When the ring is off,
    /// answers with [`PlashkaEvent::RingOff`]. When the session is still
    /// waiting for the person's screen, answers with
    /// [`PlashkaEvent::NoAccess`]. When the ring holds no frame, answers with
    /// [`PlashkaEvent::NothingToSave`]. When the otkat folder cannot be
    /// created, answers with [`PlashkaEvent::FolderNotCreated`]. When the disk
    /// has no free space, answers with [`PlashkaEvent::NoSpace`]. None of those
    /// creates a file. A ring that holds frames, can use the folder, and has
    /// room answers with [`PlashkaEvent::Saved`]; the file is written when that
    /// value is dropped. If that write breaks, the unfinished file is removed
    /// and [`PlashkaEvent::ClipDidNotSave`] is left for [`Ring::later_plashka`].
    /// That is a different plashka from [`PlashkaEvent::NoSpace`], which is the
    /// answer when the disk is already full at the press. The answer does not
    /// wait for an earlier file that is still being written. That otkat keeps
    /// the ring as it is at this bind and waits in the queue.
    pub fn bind(&mut self, chord: Bind) -> Option<PlashkaEvent> {
        if self.talk.is_some() {
            return self.bind_on_process(&chord);
        }
        if chord != self.bind {
            return None;
        }
        if self.switch == Switch::Off {
            return Some(PlashkaEvent::RingOff);
        }
        if self.waiting_for_screen || self.output_refused || self.microphone_refused {
            return Some(PlashkaEvent::NoAccess);
        }
        if self.held.is_zero() {
            return Some(PlashkaEvent::NothingToSave);
        }
        if !otkat_folder_can_be_created(&self.otkat_folder) {
            return Some(PlashkaEvent::FolderNotCreated);
        }
        if !filesystem_has_room(&self.otkat_folder) {
            return Some(PlashkaEvent::NoSpace);
        }
        let day = (self.clock)();
        let slices = self.slices_for_save();
        let duration = slices
            .iter()
            .map(|slice| slice.duration)
            .sum::<Duration>()
            .min(self.chosen_length);
        let (ticket, number) = self.queue.reserve(&self.otkat_folder);
        let name = format!("otkat{number}-{}.mp4", day.label());
        let path = self.otkat_folder.join(&name);
        let (width, height) = self.encoded_frame();
        let bitrate_mbit = self.bitrate.mbit_per_sec();
        let monitor = self.segment.as_ref().map(|segment| segment.monitor.clone());
        let cursor_in_picture = slices.iter().any(|slice| slice.cursor);
        let output_sound = slices.iter().any(|slice| slice.output_sound);
        let microphone = slices.iter().any(|slice| slice.microphone);
        let frames_per_second = if monitor.is_some() { PICTURE_RATE } else { 0 };
        Some(PlashkaEvent::Saved(SavedOtkat {
            name,
            duration,
            width,
            height,
            bitrate_mbit,
            pending: Some(PendingOtkat {
                path,
                duration,
                width,
                height,
                bitrate_mbit,
                frames_per_second,
                encoder: self.encoder,
                slices,
                ticket,
                number,
                queue: Arc::clone(&self.queue),
            }),
            process_commit: None,
            monitor,
            cursor_in_picture,
            output_sound,
            microphone,
            encoded: true,
            frames_per_second,
            encoder: self.encoder,
        }))
    }

    fn bind_on_process(&mut self, chord: &Bind) -> Option<PlashkaEvent> {
        let reply = self.say(&format!("bind\t{}", chord.text()));
        let mut parts = reply.split('\t');
        let kind = parts.next().unwrap_or("");
        match kind {
            "none" => None,
            "nothing" => Some(PlashkaEvent::NothingToSave),
            "no-access" => Some(PlashkaEvent::NoAccess),
            "folder" => Some(PlashkaEvent::FolderNotCreated),
            "no-space" => Some(PlashkaEvent::NoSpace),
            "ring-off" => Some(PlashkaEvent::RingOff),
            "clip-did-not-save" => Some(PlashkaEvent::ClipDidNotSave),
            "saved" => {
                let name = parts.next().unwrap_or("otkat.mp4").to_string();
                let secs = parts.next().unwrap_or("0").parse::<u64>().unwrap_or(0);
                let talk = self.talk.clone().expect("the ring process is attached");
                Some(PlashkaEvent::Saved(SavedOtkat {
                    name,
                    duration: Duration::from_secs(secs),
                    width: 0,
                    height: 0,
                    bitrate_mbit: 0,
                    pending: None,
                    process_commit: Some(ProcessCommit {
                        talk,
                        later: self.later_tx.clone(),
                    }),
                    monitor: None,
                    cursor_in_picture: false,
                    output_sound: false,
                    microphone: false,
                    encoded: true,
                    frames_per_second: 0,
                    encoder: Encoder::H264,
                }))
            }
            _ => None,
        }
    }

    fn forget_held(&mut self) {
        self.held = Duration::ZERO;
        self.segment = None;
        self.slices.clear();
        self.heard_output = false;
        self.heard_microphone = false;
        let _ = fs::remove_file(self.ring_folder.join("segment.mp4"));
    }

    fn trim_slices(&mut self) {
        let limit = self.chosen_length;
        loop {
            let total: Duration = self.slices.iter().map(|slice| slice.duration).sum();
            if total <= limit || self.slices.is_empty() {
                self.held = total;
                return;
            }
            let extra = total - limit;
            if self.slices[0].duration <= extra {
                self.slices.remove(0);
            } else {
                self.slices[0].duration -= extra;
            }
        }
    }

    fn rebuild_segment(&mut self) {
        let Some(monitor) = self.screen.clone() else {
            self.segment = None;
            return;
        };
        if self.slices.is_empty() {
            self.segment = None;
            return;
        }
        self.segment = Some(EncodedSegment {
            monitor,
            duration: self.held,
            frames_per_second: PICTURE_RATE,
        });
    }

    fn slices_for_save(&self) -> Vec<MomentSlice> {
        if self.slices.is_empty() {
            if self.held.is_zero() {
                Vec::new()
            } else {
                vec![MomentSlice::blank(self.held.min(self.chosen_length))]
            }
        } else {
            self.slices.clone()
        }
    }

    fn encoded_frame(&self) -> (u32, u32) {
        let Some((width, height)) = self
            .screen
            .as_ref()
            .and_then(Screen::frame)
            .or(self.known_frame)
        else {
            return STAND_IN_FRAME;
        };
        let percent = u32::from(self.resolution_share.percent());
        let width = even_pixels(width * percent / 100);
        let height = even_pixels(height * percent / 100);
        if width == 0 || height == 0 {
            STAND_IN_FRAME
        } else {
            (width, height)
        }
    }

    /// On disk, the encoded ring is a file in the ring folder. In memory, it is not.
    fn store_encoded_ring(&self) {
        if self.ring_place != RingPlace::Disk {
            return;
        }
        let path = self.ring_folder.join("segment.mp4");
        if self.held.is_zero() || self.slices.is_empty() {
            let _ = fs::remove_file(&path);
            return;
        }
        let (width, height) = self.encoded_frame();
        let job = OtkatJob {
            path,
            duration: self.held,
            width,
            height,
            bitrate_mbit: self.bitrate.mbit_per_sec(),
            frames_per_second: PICTURE_RATE,
            encoder: self.encoder,
            slices: self.slices.clone(),
        };
        if let Err(err) = encode_mp4(&job) {
            eprintln!("encoded ring was not stored: {err}");
        }
    }

    fn write_screen_answer(&self) {
        let Some(screen) = &self.screen else {
            return;
        };
        let path = screen_answer_path(&self.home);
        if let Some(parent) = path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                eprintln!("screen answer folder was not created: {err}");
                return;
            }
        }
        if let Err(err) = fs::write(&path, format!("{}\n", screen.name())) {
            eprintln!("screen answer was not remembered: {err}");
        }
    }
}

fn filesystem_has_room(path: &Path) -> bool {
    let Some(existing) = nearest_existing(path) else {
        return true;
    };
    available_blocks(existing).is_none_or(|blocks| blocks > 0)
}

fn nearest_existing(mut path: &Path) -> Option<&Path> {
    loop {
        if path.exists() {
            return Some(path);
        }
        path = path.parent()?;
    }
}

fn available_blocks(path: &Path) -> Option<u64> {
    let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut stat = unsafe { std::mem::zeroed::<Statvfs>() };
    let rc = unsafe { statvfs(c_path.as_ptr(), &mut stat) };
    if rc == 0 {
        Some(stat.f_bavail)
    } else {
        None
    }
}

#[repr(C)]
struct Statvfs {
    f_bsize: u64,
    f_frsize: u64,
    f_blocks: u64,
    f_bfree: u64,
    f_bavail: u64,
    f_files: u64,
    f_ffree: u64,
    f_favail: u64,
    f_fsid: u64,
    f_flag: u64,
    f_namemax: u64,
    f_type: u32,
    spare: [i32; 5],
}

extern "C" {
    fn statvfs(path: *const std::os::raw::c_char, buf: *mut Statvfs) -> i32;
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn ring_folder_under(home: &Path) -> PathBuf {
    home.join(".local/state/otkati/ring")
}

fn screen_answer_path(home: &Path) -> PathBuf {
    home.join(".local/state/otkati/screen-answer")
}

fn read_screen_answer(home: &Path) -> Option<Screen> {
    let text = fs::read_to_string(screen_answer_path(home)).ok()?;
    let name = text.lines().next()?.trim();
    if name.is_empty() {
        None
    } else {
        Some(Screen::new(name))
    }
}

/// The folder is created with the file, after the plashka, so this only looks.
fn otkat_folder_can_be_created(folder: &Path) -> bool {
    if folder.is_dir() {
        return true;
    }
    if folder.exists() {
        return false;
    }
    let mut ancestor = folder;
    loop {
        let Some(parent) = ancestor.parent() else {
            return false;
        };
        if parent == ancestor {
            return false;
        }
        if parent.exists() {
            return parent.is_dir() && directory_accepts_a_child(parent);
        }
        ancestor = parent;
    }
}

fn directory_accepts_a_child(path: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt;
    let Ok(c_path) = std::ffi::CString::new(path.as_os_str().as_bytes()) else {
        return false;
    };
    extern "C" {
        fn access(pathname: *const std::os::raw::c_char, mode: i32) -> i32;
    }
    const W_OK: i32 = 2;
    const X_OK: i32 = 1;
    unsafe { access(c_path.as_ptr(), W_OK | X_OK) == 0 }
}

/// The path the ring picks from the card. The person does not choose it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum VendorPath {
    Nvidia,
    Amd,
}

fn vendor_path(card: Card) -> Option<VendorPath> {
    match card {
        Card::Nvidia3060Ti | Card::Nvidia3070Ti | Card::Nvidia40Series => Some(VendorPath::Nvidia),
        Card::Amd { .. } => Some(VendorPath::Amd),
        Card::Intel { .. } | Card::NoHardwareEncoder => None,
    }
}

fn hevc_offered(card: Card) -> bool {
    match vendor_path(card) {
        Some(VendorPath::Nvidia) => true,
        Some(VendorPath::Amd) => matches!(card, Card::Amd { hevc: true, .. }),
        None => matches!(card, Card::Intel { hevc: true }),
    }
}

fn av1_offered(card: Card) -> bool {
    match vendor_path(card) {
        Some(VendorPath::Nvidia) => matches!(card, Card::Nvidia40Series),
        Some(VendorPath::Amd) => matches!(card, Card::Amd { rx_7000: true, .. }),
        None => false,
    }
}

fn next_otkat_number(folder: &Path) -> u32 {
    let mut highest = 0u32;
    let Ok(entries) = fs::read_dir(folder) else {
        return 1;
    };
    for entry in entries.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if let Some(number) = otkat_number(&name) {
            highest = highest.max(number);
        }
    }
    highest + 1
}

fn otkat_number(name: &str) -> Option<u32> {
    let rest = name.strip_prefix("otkat")?.strip_suffix(".mp4")?;
    let (number, date) = rest.split_once('-')?;
    if date.len() != 10 || number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    number.parse().ok()
}

fn system_bind_day() -> BindDay {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0);
    local_bind_day(secs)
}

fn local_bind_day(secs: i64) -> BindDay {
    #[repr(C)]
    struct Tm {
        tm_sec: i32,
        tm_min: i32,
        tm_hour: i32,
        tm_mday: i32,
        tm_mon: i32,
        tm_year: i32,
        tm_wday: i32,
        tm_yday: i32,
        tm_isdst: i32,
        tm_gmtoff: i64,
        tm_zone: *const std::os::raw::c_char,
    }

    extern "C" {
        fn localtime_r(timep: *const i64, result: *mut Tm) -> *mut Tm;
    }

    unsafe {
        let mut tm = std::mem::zeroed::<Tm>();
        if localtime_r(&secs, &mut tm).is_null() {
            return BindDay::new(1970, 1, 1);
        }
        BindDay::new(tm.tm_year + 1900, (tm.tm_mon + 1) as u8, tm.tm_mday as u8)
    }
}
