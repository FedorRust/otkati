use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::time::Duration;

use crate::{read_screen_answer, Bind, BindDay, PlashkaEvent, Ring, Screen};

/// The `otkati-ring` process. It is the ring: it remembers the screen answer,
/// holds the encoded minutes, and a bind here saves what it holds.
pub fn run_ring_process() {
    let folder = std::env::var("OTKATI_OTKAT_FOLDER")
        .map(PathBuf::from)
        .unwrap_or_else(|_| Ring::default_otkat_folder());
    let home = std::env::var("OTKATI_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| crate::home_dir());
    let ready = std::env::var_os("OTKATI_READY").is_some();
    let day = std::env::var("OTKATI_BIND_DAY").ok();
    let clock = move || match day.as_deref() {
        Some(text) => parse_bind_day(text),
        None => crate::system_bind_day(),
    };
    let mut ring = Ring::with_clock(folder, clock);
    if !ready {
        if let Some(screen) = read_screen_answer(&home) {
            ring.remember_screen(screen);
        } else {
            ring.begin_waiting();
        }
    }
    if std::env::var_os("OTKATI_HOME").is_some() {
        ring.persist_at(home);
    }
    if std::env::var_os("OTKATI_SERVE").is_some() {
        serve(ring);
    } else {
        // The answer and any minutes already loaded stay in this process.
        std::thread::park();
    }
}

fn serve(mut ring: Ring) {
    let stdin = std::io::stdin();
    let mut reader = stdin.lock();
    let mut line = String::new();
    let mut pending: Vec<PlashkaEvent> = Vec::new();
    loop {
        line.clear();
        let read = reader.read_line(&mut line).unwrap_or(0);
        if read == 0 {
            break;
        }
        let reply = dispatch(&mut ring, &mut pending, line.trim_end());
        println!("{reply}");
        let _ = std::io::stdout().flush();
    }
}

fn dispatch(ring: &mut Ring, pending: &mut Vec<PlashkaEvent>, line: &str) -> String {
    let (command, arg) = line.split_once('\t').unwrap_or((line, ""));
    match command {
        "query" => format!(
            "{}\t{}\t{}",
            if ring.asks_which_screen() { "1" } else { "0" },
            ring.monitor().map(Screen::name).unwrap_or("-"),
            ring.held.as_secs()
        ),
        "hold" => {
            let secs = arg.parse::<u64>().unwrap_or(0);
            ring.hold(Duration::from_secs(secs));
            "ok".to_string()
        }
        "remember" => {
            if !arg.is_empty() {
                ring.remember_screen(Screen::new(arg));
            }
            "ok".to_string()
        }
        "bind" => bind_line(ring, pending, arg),
        "commit" => commit_pending(ring, pending),
        _ => "none".to_string(),
    }
}

fn bind_line(ring: &mut Ring, pending: &mut Vec<PlashkaEvent>, chord: &str) -> String {
    match ring.bind(Bind::new(chord)) {
        None => "none".to_string(),
        Some(PlashkaEvent::NothingToSave) => "nothing".to_string(),
        Some(PlashkaEvent::NoAccess) => "no-access".to_string(),
        Some(PlashkaEvent::FolderNotCreated) => "folder".to_string(),
        Some(PlashkaEvent::NoSpace) => "no-space".to_string(),
        Some(PlashkaEvent::ClipDidNotSave) => "clip-did-not-save".to_string(),
        Some(PlashkaEvent::RingOff) => "ring-off".to_string(),
        Some(PlashkaEvent::Saved(saved)) => {
            let line = format!("saved\t{}\t{}", saved.name(), saved.duration().as_secs());
            pending.push(PlashkaEvent::Saved(saved));
            line
        }
    }
}

fn commit_pending(ring: &mut Ring, pending: &mut Vec<PlashkaEvent>) -> String {
    if pending.is_empty() {
        return "ok".to_string();
    }
    drop(pending.remove(0));
    match ring.later_plashka() {
        Some(PlashkaEvent::ClipDidNotSave) => "clip-did-not-save".to_string(),
        _ => "ok".to_string(),
    }
}

fn parse_bind_day(text: &str) -> BindDay {
    let mut parts = text.split('-');
    let year = parts.next().unwrap_or("1970").parse().unwrap_or(1970);
    let month = parts.next().unwrap_or("1").parse().unwrap_or(1);
    let day = parts.next().unwrap_or("1").parse().unwrap_or(1);
    BindDay::new(year, month, day)
}
