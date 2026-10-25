use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

/// A screen the person can give the program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screen {
    name: String,
}

impl Screen {
    /// The screen the person chose.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// How this screen is told apart from another.
    pub fn name(&self) -> &str {
        &self.name
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
}

impl PartialEq for PlashkaEvent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NothingToSave, Self::NothingToSave) => true,
            (Self::NoAccess, Self::NoAccess) => true,
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
/// in the otkat folder when this value is dropped.
#[derive(Debug)]
pub struct SavedOtkat {
    name: String,
    duration: Duration,
    pending: Option<PendingOtkat>,
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
}

impl Drop for SavedOtkat {
    fn drop(&mut self) {
        if let Some(pending) = self.pending.take() {
            if let Err(err) = pending.write() {
                eprintln!("otkat was not written: {err}");
            }
        }
    }
}

#[derive(Debug)]
struct PendingOtkat {
    path: PathBuf,
    duration: Duration,
}

impl PendingOtkat {
    fn write(self) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let seconds = self.duration.as_secs();
        let output = Command::new("ffmpeg")
            .arg("-y")
            .arg("-loglevel")
            .arg("error")
            .arg("-f")
            .arg("lavfi")
            .arg("-i")
            .arg(format!("color=c=black:s=16x16:r=1:d={seconds}"))
            .arg("-c:v")
            .arg("libx264")
            .arg("-preset")
            .arg("ultrafast")
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg("-an")
            .arg(&self.path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()?;
        if output.status.success() {
            Ok(())
        } else {
            let detail = String::from_utf8_lossy(&output.stderr);
            Err(std::io::Error::other(format!(
                "ffmpeg exited with {}: {detail}",
                output.status
            )))
        }
    }
}

/// The length the ring aims to hold until the length knob exists: 10 minutes.
const CHOSEN_LENGTH: Duration = Duration::from_secs(10 * 60);

/// Encoded seconds the ring holds, and the bind that asks to save an otkat.
///
/// A newly created ring holds no frame and listens for [`Ring::default_bind`]
/// until the face hands it a new one. A session entered with no remembered
/// screen is still waiting for the person's answer.
pub struct Ring {
    otkat_folder: PathBuf,
    held: Duration,
    clock: Box<dyn Fn() -> BindDay + Send>,
    waiting_for_screen: bool,
    screen: Option<Screen>,
    bind: Bind,
    card: Option<Card>,
    encoder: Encoder,
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

    /// `otkat_folder` is where a saved otkat is written.
    pub fn new(otkat_folder: impl Into<PathBuf>) -> Self {
        Self::with_clock(otkat_folder, system_bind_day)
    }

    /// `clock` answers the bind day. [`Ring::new`] uses the local calendar day.
    pub fn with_clock(
        otkat_folder: impl Into<PathBuf>,
        clock: impl Fn() -> BindDay + Send + 'static,
    ) -> Self {
        Self {
            otkat_folder: otkat_folder.into(),
            held: Duration::ZERO,
            clock: Box::new(clock),
            waiting_for_screen: false,
            screen: None,
            bind: Self::default_bind(),
            card: None,
            encoder: Encoder::H264,
        }
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
        Self {
            otkat_folder: otkat_folder.into(),
            held: Duration::ZERO,
            clock: Box::new(system_bind_day),
            waiting_for_screen,
            screen: remembered,
            bind: Self::default_bind(),
            card: None,
            encoder: Encoder::H264,
        }
    }

    /// Whether this session still asks which screen to give the program.
    pub fn asks_which_screen(&self) -> bool {
        self.waiting_for_screen
    }

    /// The person answered with this screen. It is the ring's monitor, and
    /// the session stops asking.
    pub fn remember_screen(&mut self, screen: Screen) {
        self.screen = Some(screen);
        self.waiting_for_screen = false;
    }

    /// Offer a screen to the ring.
    ///
    /// Until the person has answered, the screen does not enter the ring
    /// and does not become the ring's monitor.
    pub fn offer_screen(&mut self, _screen: Screen) {}

    /// The ring's monitor: the screen from the person's answer.
    pub fn monitor(&self) -> Option<&Screen> {
        self.screen.as_ref()
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
    pub fn see_card(&mut self, card: Card) {
        self.card = Some(card);
    }

    /// Encoder items this card can do, in knob order.
    ///
    /// Until the ring has seen a card, the only item is H.264.
    pub fn offered_encoders(&self) -> Vec<Encoder> {
        let mut offered = vec![Encoder::H264];
        if matches!(
            self.card,
            Some(Card::Nvidia3060Ti | Card::Nvidia3070Ti)
        ) {
            offered.push(Encoder::Hevc);
        }
        offered
    }

    /// Output sound arrived. It does not ask which screen to give.
    pub fn output_sound(&mut self) {}

    /// The microphone arrived. It does not ask which screen to give.
    pub fn microphone(&mut self) {}

    /// The ring holds `held` of encoded time.
    ///
    /// Live capture from the monitor fills this in a later ticket.
    pub fn hold(&mut self, held: Duration) {
        self.held = held;
    }

    /// The face hands the ring a new bind.
    ///
    /// The ring saves on that chord, and the previous chord no longer does.
    /// No Hyprland config is read or written.
    pub fn take_bind(&mut self, bind: Bind) {
        self.bind = bind;
    }

    /// `chord` reached the ring.
    ///
    /// When it is not the ring's bind, nothing is saved. When the session is
    /// still waiting for the person's screen, answers with
    /// [`PlashkaEvent::NoAccess`]. When the ring holds no frame, answers with
    /// [`PlashkaEvent::NothingToSave`]. Neither of those creates a file.
    /// A ring that holds frames answers with [`PlashkaEvent::Saved`]; the file
    /// is written when that value is dropped.
    pub fn bind(&mut self, chord: Bind) -> Option<PlashkaEvent> {
        if chord != self.bind {
            return None;
        }
        if self.waiting_for_screen {
            return Some(PlashkaEvent::NoAccess);
        }
        if self.held.is_zero() {
            return Some(PlashkaEvent::NothingToSave);
        }
        let day = (self.clock)();
        let duration = self.held.min(CHOSEN_LENGTH);
        let number = next_otkat_number(&self.otkat_folder);
        let name = format!("otkat{number}-{}.mp4", day.label());
        let path = self.otkat_folder.join(&name);
        Some(PlashkaEvent::Saved(SavedOtkat {
            name,
            duration,
            pending: Some(PendingOtkat { path, duration }),
        }))
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
