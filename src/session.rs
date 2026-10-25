use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// How the ring enters a Hyprland session.
///
/// The session starts the ring. The person does not open it first.
pub struct SessionAutostart {
    exec_start: String,
    starts_with_graphical_session: bool,
}

impl SessionAutostart {
    /// The autostart shipped with the program.
    pub fn shipped() -> Self {
        let path = shipped_unit_path();
        let text = fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("session autostart is missing at {}: {err}", path.display())
        });
        parse_unit(&text)
    }

    /// Log in to a Hyprland session.
    ///
    /// The ring is already running. The person does not open it.
    /// `program_dir` is where the session finds the program the autostart names.
    pub fn log_in(&self, program_dir: impl AsRef<Path>) -> HyprlandSession {
        let ring = if self.starts_with_graphical_session {
            let program = resolve_program(&self.exec_start, program_dir.as_ref());
            Some(spawn_in_its_own_group(&program))
        } else {
            None
        };
        HyprlandSession { ring }
    }
}

/// A Hyprland session after login.
pub struct HyprlandSession {
    ring: Option<Child>,
}

impl HyprlandSession {
    /// Whether the ring process the session started is still running.
    pub fn ring_is_running(&mut self) -> bool {
        let Some(child) = self.ring.as_mut() else {
            return false;
        };
        matches!(child.try_wait(), Ok(None))
    }
}

impl Drop for HyprlandSession {
    fn drop(&mut self) {
        if let Some(mut ring) = self.ring.take() {
            let _ = ring.kill();
            let _ = ring.wait();
        }
    }
}

fn shipped_unit_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("systemd/user/otkati-ring.service")
}

fn parse_unit(text: &str) -> SessionAutostart {
    let mut section = String::new();
    let mut exec_start = String::new();
    let mut starts_with_graphical_session = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(name) = line
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            section = name.to_string();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        if section == "Service" && key == "ExecStart" {
            exec_start = value.to_string();
        }
        if section == "Install" && key == "WantedBy" {
            starts_with_graphical_session |= value
                .split_whitespace()
                .any(|target| target == "graphical-session.target");
        }
    }
    SessionAutostart {
        exec_start,
        starts_with_graphical_session,
    }
}

fn resolve_program(exec_start: &str, program_dir: &Path) -> PathBuf {
    let program = exec_start.split_whitespace().next().unwrap_or("");
    let path = PathBuf::from(program);
    if path.is_absolute() {
        path
    } else {
        program_dir.join(path)
    }
}

fn spawn_in_its_own_group(program: &Path) -> Child {
    let mut command = Command::new(program);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    command
        .spawn()
        .unwrap_or_else(|err| panic!("session could not start {}: {err}", program.display()))
}
