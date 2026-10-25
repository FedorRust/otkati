use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The key combination that asks the ring to save an otkat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bind {
    text: &'static str,
}

impl Bind {
    /// How the bind is written, for example `Alt+Z`.
    pub fn text(self) -> &'static str {
        self.text
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
}

impl PartialEq for PlashkaEvent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NothingToSave, Self::NothingToSave) => true,
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

/// Encoded seconds the ring holds, and the bind that asks to save an otkat.
///
/// A newly created ring holds no frame.
pub struct Ring {
    otkat_folder: PathBuf,
    held: Duration,
    clock: Box<dyn Fn() -> BindDay + Send>,
}

impl Ring {
    /// The bind a new ring listens for, before anyone changes it.
    pub fn default_bind() -> Bind {
        Bind { text: "Alt+Z" }
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
        }
    }

    /// The ring holds `held` of encoded time.
    ///
    /// Live capture from the monitor fills this in a later ticket.
    pub fn hold(&mut self, held: Duration) {
        self.held = held;
    }

    /// The bind was pressed.
    ///
    /// An empty ring answers with [`PlashkaEvent::NothingToSave`] and does not
    /// create a file in the otkat folder. A ring that holds frames answers with
    /// [`PlashkaEvent::Saved`]; the file is written when that value is dropped.
    pub fn bind(&mut self) -> PlashkaEvent {
        if self.held.is_zero() {
            return PlashkaEvent::NothingToSave;
        }
        let day = (self.clock)();
        let duration = self.held;
        let name = format!("otkat1-{}.mp4", day.label());
        let path = self.otkat_folder.join(&name);
        PlashkaEvent::Saved(SavedOtkat {
            name,
            duration,
            pending: Some(PendingOtkat { path, duration }),
        })
    }
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
