use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
/// in the otkat folder when this value is dropped. A later bind while this
/// file is still unwritten keeps its own name and moment; the queue writes
/// one file at a time, in bind order.
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
            let queue = Arc::clone(&pending.queue);
            queue.write_when_its_turn(pending);
        }
    }
}

struct PendingOtkat {
    path: PathBuf,
    duration: Duration,
    ticket: u64,
    number: u32,
    queue: Arc<OtkatQueue>,
}

impl std::fmt::Debug for PendingOtkat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingOtkat")
            .field("path", &self.path)
            .field("duration", &self.duration)
            .field("ticket", &self.ticket)
            .field("number", &self.number)
            .finish_non_exhaustive()
    }
}

impl PendingOtkat {
    fn encode(&self) -> std::io::Result<()> {
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

/// Otkats already taken from the ring and not yet finished as files.
struct OtkatQueue {
    inner: Mutex<QueueInner>,
    turn: Condvar,
}

struct QueueInner {
    next_ticket: u64,
    next_to_write: u64,
    outstanding: BTreeSet<u32>,
}

impl OtkatQueue {
    fn new() -> Self {
        Self {
            inner: Mutex::new(QueueInner {
                next_ticket: 1,
                next_to_write: 1,
                outstanding: BTreeSet::new(),
            }),
            turn: Condvar::new(),
        }
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
        let result = pending.encode();
        {
            let mut inner = self.inner.lock().unwrap();
            inner.outstanding.remove(&pending.number);
            inner.next_to_write += 1;
            self.turn.notify_all();
        }
        if let Err(err) = result {
            eprintln!("otkat was not written: {err}");
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
    queue: Arc<OtkatQueue>,
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
            queue: Arc::new(OtkatQueue::new()),
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
            queue: Arc::new(OtkatQueue::new()),
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
    /// is written when that value is dropped. The answer does not wait for an
    /// earlier file that is still being written. That otkat keeps the ring as
    /// it is at this bind and waits in the queue.
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
        let (ticket, number) = self.queue.reserve(&self.otkat_folder);
        let name = format!("otkat{number}-{}.mp4", day.label());
        let path = self.otkat_folder.join(&name);
        Some(PlashkaEvent::Saved(SavedOtkat {
            name,
            duration,
            pending: Some(PendingOtkat {
                path,
                duration,
                ticket,
                number,
                queue: Arc::clone(&self.queue),
            }),
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
