use std::ffi::CString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Where the encoded ring lies until an otkat is saved from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingPlace {
    /// The ring stays in memory. This is the place until the person chooses disk.
    Memory,
    /// The encoded ring lies in the ring folder.
    Disk,
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
    /// The disk has no space, so no finished otkat is written.
    NoSpace,
    /// The ring is off, so the bind saves nothing.
    RingOff,
}

impl PartialEq for PlashkaEvent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NothingToSave, Self::NothingToSave) => true,
            (Self::NoAccess, Self::NoAccess) => true,
            (Self::NoSpace, Self::NoSpace) => true,
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
            if detail.contains("No space left on device") {
                let _ = fs::remove_file(&self.path);
            }
            Err(std::io::Error::other(format!(
                "ffmpeg exited with {}: {detail}",
                output.status
            )))
        }
    }
}

/// Ten minutes, until the face hands a different length.
const DEFAULT_LENGTH: Duration = Duration::from_secs(10 * 60);

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
    bind: Bind,
    ring_place: RingPlace,
    ring_folder: PathBuf,
    switch: Switch,
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
        ring.ring_folder = ring_folder_under(&home.into());
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
        self.held = Duration::ZERO;
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
        Self {
            otkat_folder: otkat_folder.into(),
            held: Duration::ZERO,
            chosen_length: DEFAULT_LENGTH,
            clock: Box::new(clock),
            waiting_for_screen: false,
            screen: None,
            bind: Self::default_bind(),
            ring_place: RingPlace::Memory,
            ring_folder: ring_folder_under(&home_dir()),
            switch: Switch::On,
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
            chosen_length: DEFAULT_LENGTH,
            clock: Box::new(system_bind_day),
            waiting_for_screen,
            screen: remembered,
            bind: Self::default_bind(),
            ring_place: RingPlace::Memory,
            ring_folder: ring_folder_under(&home_dir()),
            switch: Switch::On,
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
    /// While the switch is off the ring is not gathering, so this leaves the
    /// ring unchanged. Live capture from the monitor fills this in a later ticket.
    pub fn hold(&mut self, held: Duration) {
        if self.switch == Switch::Off {
            return;
        }
        self.held = held;
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
            self.held = Duration::ZERO;
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
    /// When it is not the ring's bind, nothing is saved. When the ring is off,
    /// answers with [`PlashkaEvent::RingOff`]. When the session is still
    /// waiting for the person's screen, answers with
    /// [`PlashkaEvent::NoAccess`]. When the ring holds no frame, answers with
    /// [`PlashkaEvent::NothingToSave`]. When the disk has no free space, answers
    /// with [`PlashkaEvent::NoSpace`]. None of those creates a file.
    /// A ring that holds frames and has room answers with [`PlashkaEvent::Saved`];
    /// the file is written when that value is dropped.
    pub fn bind(&mut self, chord: Bind) -> Option<PlashkaEvent> {
        if chord != self.bind {
            return None;
        }
        if self.switch == Switch::Off {
            return Some(PlashkaEvent::RingOff);
        }
        if self.waiting_for_screen {
            return Some(PlashkaEvent::NoAccess);
        }
        if self.held.is_zero() {
            return Some(PlashkaEvent::NothingToSave);
        }
        if !filesystem_has_room(&self.otkat_folder) {
            return Some(PlashkaEvent::NoSpace);
        }
        let day = (self.clock)();
        let duration = self.held.min(self.chosen_length);
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
