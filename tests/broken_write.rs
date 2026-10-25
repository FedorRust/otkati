use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use otkati::{BindDay, PlashkaEvent, Ring};

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
        let path = std::env::temp_dir().join(format!("otkati-broken-write-{tick}-{n}"));
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
fn a_write_that_breaks_after_the_success_plashka_shows_clip_did_not_save() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    let (created_tx, created_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Arc::new(Mutex::new(release_rx));
    ring.write_otkat_with(move |path, _duration| {
        fs::write(path, b"unfinished")?;
        created_tx.send(()).expect("the test is still waiting");
        let _ = release_rx.lock().unwrap().recv();
        Err(io::Error::other("write broke"))
    });
    ring.hold(Duration::from_secs(30));

    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected the success plashka at the bind, got {event:?}");
    };
    assert_eq!(saved.name(), "otkat1-2026-10-25.mp4");
    assert_eq!(saved.duration(), Duration::from_secs(30));
    let path = folder.path.join("otkat1-2026-10-25.mp4");
    assert!(
        !path.exists(),
        "the success plashka comes out before the file is in the folder"
    );

    let write = thread::spawn(move || drop(event));
    created_rx.recv().unwrap();
    assert!(
        path.exists(),
        "the broken write leaves an unfinished file before it fails"
    );
    assert_eq!(
        ring.later_plashka(),
        None,
        "the second plashka waits until the write has broken"
    );

    release_tx.send(()).unwrap();
    write.join().unwrap();

    assert_eq!(ring.later_plashka(), Some(PlashkaEvent::ClipDidNotSave));
    assert_eq!(
        ring.later_plashka(),
        None,
        "one broken write is one plashka"
    );
    assert!(
        !path.exists(),
        "the unfinished file disappears from the otkat folder"
    );
}
