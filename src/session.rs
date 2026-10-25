use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::{BindDay, Ring, RingTalk};

/// How the ring enters a Hyprland session.
///
/// The session starts the ring. The person does not open it first, and
/// Serpantinum does not own the process.
pub struct SessionAutostart {
    exec_start: String,
    starts_with_graphical_session: bool,
    tied_to_shell: bool,
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
        let folder = Ring::default_otkat_folder();
        let ring = Ring::new(folder.clone());
        self.open(program_dir, ring, serve_env(&folder, None, None, true))
    }

    /// Log in, with `clock` answering the bind day.
    ///
    /// `otkat_folder` is where a bind writes. Minutes the ring already holds
    /// are still there after Serpantinum restarts.
    pub fn log_in_with_clock(
        &self,
        otkat_folder: impl Into<PathBuf>,
        program_dir: impl AsRef<Path>,
        clock: impl Fn() -> BindDay + Send + 'static,
    ) -> HyprlandSession {
        let folder = otkat_folder.into();
        let day = clock();
        let ring = Ring::with_clock(folder.clone(), move || day);
        self.open(
            program_dir,
            ring,
            serve_env(&folder, None, Some(day), true),
        )
    }

    /// Log in to a ring that asks which screen until the process has remembered one.
    ///
    /// `home` is where that answer is kept. The next session with the same home
    /// does not ask again.
    pub fn log_in_at(
        &self,
        otkat_folder: impl Into<PathBuf>,
        home: impl Into<PathBuf>,
        program_dir: impl AsRef<Path>,
        clock: impl Fn() -> BindDay + Send + 'static,
    ) -> HyprlandSession {
        let folder = otkat_folder.into();
        let home = home.into();
        let day = clock();
        let ring = Ring::enter_session(folder.clone(), None);
        self.open(
            program_dir,
            ring,
            serve_env(&folder, Some(&home), Some(day), false),
        )
    }

    fn open(
        &self,
        program_dir: impl AsRef<Path>,
        mut ring: Ring,
        env: Vec<(String, String)>,
    ) -> HyprlandSession {
        let shell = spawn_shell();
        let ring_process = if self.starts_with_graphical_session {
            let program = resolve_program(&self.exec_start, program_dir.as_ref());
            let group = if self.tied_to_shell {
                Some(shell.id())
            } else {
                None
            };
            let mut child = spawn_ring(&program, group, &env);
            let stdin = child.stdin.take().expect("ring stdin");
            let stdout = BufReader::new(child.stdout.take().expect("ring stdout"));
            let pipe = Arc::new(Mutex::new(ProcessPipe { stdin, stdout }));
            let talk_pipe = Arc::clone(&pipe);
            let talk: RingTalk = Arc::new(Mutex::new(move |line: &str| {
                let mut pipe = talk_pipe.lock().unwrap();
                writeln!(pipe.stdin, "{line}").expect("ring process stdin");
                pipe.stdin.flush().expect("ring process stdin");
                let mut reply = String::new();
                let read = pipe
                    .stdout
                    .read_line(&mut reply)
                    .expect("ring process stdout");
                if read == 0 {
                    panic!("ring process closed stdout answering: {line}");
                }
                reply.trim_end().to_string()
            }));
            ring.attach_process(talk);
            Some(child)
        } else {
            None
        };
        HyprlandSession {
            ring_process,
            shell,
            ring,
        }
    }
}

/// A Hyprland session after login.
///
/// Serpantinum is running too. The ring is not its child.
pub struct HyprlandSession {
    ring_process: Option<Child>,
    shell: Child,
    ring: Ring,
}

impl HyprlandSession {
    /// Whether the ring process the session started is still running.
    pub fn ring_is_running(&mut self) -> bool {
        let Some(child) = self.ring_process.as_mut() else {
            return false;
        };
        matches!(child.try_wait(), Ok(None))
    }

    /// The ring process login started.
    pub fn ring_pid(&self) -> u32 {
        self.ring_process
            .as_ref()
            .expect("the ring is running")
            .id()
    }

    /// Whether that ring process was started by Serpantinum.
    pub fn ring_is_child_of_shell(&self) -> bool {
        let Some(ring) = self.ring_process.as_ref() else {
            return false;
        };
        parent_pid(ring.id()) == self.shell.id()
    }

    /// The ring this session is holding. A bind saves from the ring process.
    pub fn ring(&mut self) -> &mut Ring {
        self.ring.pull_from_process();
        &mut self.ring
    }

    /// Restart Serpantinum.
    ///
    /// The shell process and anything it started go away. The ring stays
    /// the process login started, unless the autostart tied it to the shell.
    /// Minutes already held stay with that process, so a later bind can save them.
    pub fn restart_shell(&mut self) {
        let shell_pid = self.shell.id();
        let children = child_pids(shell_pid);
        kill_group(shell_pid);
        for pid in children {
            kill_pid(pid);
        }
        let _ = self.shell.wait();
        let alive = self
            .ring_process
            .as_mut()
            .is_some_and(|ring| matches!(ring.try_wait(), Ok(None)));
        if !alive {
            self.ring.release_process();
        }
        self.shell = spawn_shell();
    }
}

impl Drop for HyprlandSession {
    fn drop(&mut self) {
        if let Some(mut ring) = self.ring_process.take() {
            let _ = ring.kill();
            let _ = ring.wait();
        }
        kill_group(self.shell.id());
        let _ = self.shell.wait();
    }
}

fn shipped_unit_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("systemd/user/otkati-ring.service")
}

fn parse_unit(text: &str) -> SessionAutostart {
    let mut section = String::new();
    let mut exec_start = String::new();
    let mut starts_with_graphical_session = false;
    let mut tied_to_shell = false;
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
        if section == "Unit" && (key == "PartOf" || key == "BindsTo") {
            tied_to_shell |= value_ties_to_shell(value);
        }
    }
    SessionAutostart {
        exec_start,
        starts_with_graphical_session,
        tied_to_shell,
    }
}

fn value_ties_to_shell(value: &str) -> bool {
    value.split_whitespace().any(|target| {
        let target = target.to_ascii_lowercase();
        target.contains("serpantinum") || target.contains("shell")
    })
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

fn spawn_shell() -> Child {
    let mut command = Command::new("sleep");
    command
        .arg("infinity")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    command.spawn().expect("session starts Serpantinum")
}

struct ProcessPipe {
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

fn serve_env(
    folder: &Path,
    home: Option<&Path>,
    day: Option<BindDay>,
    ready: bool,
) -> Vec<(String, String)> {
    let mut env = vec![
        ("OTKATI_SERVE".to_string(), "1".to_string()),
        (
            "OTKATI_OTKAT_FOLDER".to_string(),
            folder.display().to_string(),
        ),
    ];
    if ready {
        env.push(("OTKATI_READY".to_string(), "1".to_string()));
    }
    if let Some(home) = home {
        env.push(("OTKATI_HOME".to_string(), home.display().to_string()));
    }
    if let Some(day) = day {
        env.push(("OTKATI_BIND_DAY".to_string(), day.label()));
    }
    env
}

/// `group` is the shell's process group when the autostart ties the ring to
/// Serpantinum. Otherwise the ring gets a group of its own.
fn spawn_ring(program: &Path, group: Option<u32>, env: &[(String, String)]) -> Child {
    let mut command = Command::new(program);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .process_group(group.map(|pid| pid as i32).unwrap_or(0));
    for (key, value) in env {
        command.env(key, value);
    }
    command
        .spawn()
        .unwrap_or_else(|err| panic!("session could not start {}: {err}", program.display()))
}

fn child_pids(parent: u32) -> Vec<i32> {
    let mut children = Vec::new();
    let Ok(entries) = fs::read_dir("/proc") else {
        return children;
    };
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        let Ok(pid) = name.parse::<i32>() else {
            continue;
        };
        if parent_pid(pid as u32) == parent {
            children.push(pid);
        }
    }
    children
}

fn parent_pid(pid: u32) -> u32 {
    let Ok(status) = fs::read_to_string(format!("/proc/{pid}/status")) else {
        return 0;
    };
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("PPid:") {
            return rest.trim().parse().unwrap_or(0);
        }
    }
    0
}

const SIGKILL: i32 = 9;

fn kill_pid(pid: i32) {
    unsafe { kill(pid, SIGKILL) };
}

fn kill_group(pid: u32) {
    unsafe { kill(-(pid as i32), SIGKILL) };
}

extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}
