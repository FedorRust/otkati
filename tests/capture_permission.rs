use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use otkati::{PlashkaEvent, Ring, Screen};

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
        let path = std::env::temp_dir().join(format!("otkati-capture-{tick}-{n}"));
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
fn bind_before_the_screen_answer_reports_no_access_and_writes_no_file() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::enter_session(folder.path.clone(), None);

    let event = ring.bind();

    assert_eq!(event, PlashkaEvent::NoAccess);
    let written: Vec<_> = fs::read_dir(&folder.path).unwrap().collect();
    assert!(
        written.is_empty(),
        "a ring with no screen answer must not put a file in the otkat folder"
    );
}

#[test]
fn until_the_person_answers_the_offered_screen_stays_out_of_the_ring() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::enter_session(folder.path.clone(), None);

    ring.offer_screen(Screen::new("Alexey's screen"));

    assert!(ring.monitor().is_none());
    assert_eq!(ring.bind(), PlashkaEvent::NoAccess);
    let written: Vec<_> = fs::read_dir(&folder.path).unwrap().collect();
    assert!(
        written.is_empty(),
        "an offered screen is not an otkat while the person has not answered"
    );
}

#[test]
fn the_answered_screen_is_remembered_for_the_next_session() {
    let folder = OtkatFolder::new();
    let screen = Screen::new("Alexey's screen");

    let mut first = Ring::enter_session(folder.path.clone(), None);
    assert!(first.asks_which_screen());
    first.remember_screen(screen.clone());
    assert!(!first.asks_which_screen());
    assert_eq!(first.monitor(), Some(&screen));

    let next = Ring::enter_session(folder.path.clone(), Some(screen.clone()));
    assert!(!next.asks_which_screen());
    assert_eq!(next.monitor(), Some(&screen));
}

#[test]
fn a_remembered_screen_with_no_frame_still_reports_nothing_to_save() {
    let folder = OtkatFolder::new();
    let screen = Screen::new("Alexey's screen");
    let mut ring = Ring::enter_session(folder.path.clone(), Some(screen));

    let event = ring.bind();

    assert_eq!(event, PlashkaEvent::NothingToSave);
    let written: Vec<_> = fs::read_dir(&folder.path).unwrap().collect();
    assert!(
        written.is_empty(),
        "a remembered screen with no frame must not put a file in the otkat folder"
    );
}

#[test]
fn output_sound_and_microphone_do_not_raise_the_screen_question() {
    let folder = OtkatFolder::new();
    let screen = Screen::new("Alexey's screen");
    let mut ring = Ring::enter_session(folder.path.clone(), Some(screen.clone()));

    ring.output_sound();
    ring.microphone();

    assert!(!ring.asks_which_screen());
    assert_eq!(ring.monitor(), Some(&screen));
}

#[test]
fn output_sound_and_microphone_do_not_answer_the_screen_question() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::enter_session(folder.path.clone(), None);

    ring.output_sound();
    ring.microphone();

    assert!(ring.asks_which_screen());
    assert!(ring.monitor().is_none());
    assert_eq!(ring.bind(), PlashkaEvent::NoAccess);
}
