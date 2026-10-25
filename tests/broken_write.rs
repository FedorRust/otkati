use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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

struct OpenWrite {
    entered: Mutex<usize>,
    entered_cv: Condvar,
    permits: Mutex<usize>,
    permits_cv: Condvar,
    broken_present_when_next_started: Mutex<Option<bool>>,
}

impl OpenWrite {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            entered: Mutex::new(0),
            entered_cv: Condvar::new(),
            permits: Mutex::new(0),
            permits_cv: Condvar::new(),
            broken_present_when_next_started: Mutex::new(None),
        })
    }

    fn wait_until_entered(&self, count: usize) {
        let mut entered = self.entered.lock().unwrap();
        while *entered < count {
            entered = self.entered_cv.wait(entered).unwrap();
        }
    }

    fn entered_within(&self, count: usize, limit: Duration) -> bool {
        let start = Instant::now();
        let mut entered = self.entered.lock().unwrap();
        while *entered < count {
            let left = limit.saturating_sub(start.elapsed());
            if left.is_zero() {
                return *entered >= count;
            }
            let (next, timeout) = self.entered_cv.wait_timeout(entered, left).unwrap();
            entered = next;
            if timeout.timed_out() && *entered < count {
                return false;
            }
        }
        true
    }

    fn mark_entered(&self) {
        let mut entered = self.entered.lock().unwrap();
        *entered += 1;
        self.entered_cv.notify_all();
    }

    fn wait_permit(&self) {
        let mut permits = self.permits.lock().unwrap();
        while *permits == 0 {
            permits = self.permits_cv.wait(permits).unwrap();
        }
        *permits -= 1;
    }

    fn let_one_finish(&self) {
        let mut permits = self.permits.lock().unwrap();
        *permits += 1;
        self.permits_cv.notify_all();
    }
}

#[test]
fn the_next_otkat_starts_after_the_broken_file_is_gone() {
    let folder = OtkatFolder::new();
    let broken = folder.path.join("otkat1-2026-10-25.mp4");
    let next = folder.path.join("otkat2-2026-10-25.mp4");
    let gate = OpenWrite::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    let writer_gate = Arc::clone(&gate);
    let broken_for_writer = broken.clone();
    let next_for_writer = next.clone();
    ring.write_otkat_with(move |path, duration| {
        if path == next_for_writer {
            *writer_gate.broken_present_when_next_started.lock().unwrap() =
                Some(broken_for_writer.exists());
            writer_gate.mark_entered();
            writer_gate.wait_permit();
            fs::write(path, duration.as_secs().to_string())?;
            return Ok(());
        }
        fs::write(path, b"unfinished")?;
        writer_gate.mark_entered();
        writer_gate.wait_permit();
        Err(io::Error::other("write broke"))
    });
    ring.hold(Duration::from_secs(120));

    let first = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &first else {
        panic!("expected the success plashka at the bind, got {first:?}");
    };
    assert_eq!(saved.name(), "otkat1-2026-10-25.mp4");
    assert_eq!(saved.duration(), Duration::from_secs(120));
    assert!(!broken.exists());

    let first_write = thread::spawn(move || drop(first));
    gate.wait_until_entered(1);
    assert!(
        broken.exists(),
        "the broken write leaves an unfinished file before it fails"
    );

    ring.hold(Duration::from_secs(45));
    let second = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &second else {
        panic!("expected the success plashka at the second bind, got {second:?}");
    };
    assert_eq!(saved.name(), "otkat2-2026-10-25.mp4");
    assert_eq!(saved.duration(), Duration::from_secs(45));
    assert!(!next.exists());

    let second_write = thread::spawn(move || drop(second));
    assert!(
        !gate.entered_within(2, Duration::from_millis(200)),
        "the next otkat waits while the broken one is still being written"
    );
    assert!(broken.exists());

    gate.let_one_finish();
    assert!(
        gate.entered_within(2, Duration::from_secs(2)),
        "one broken write does not stop the queue"
    );
    assert_eq!(
        *gate.broken_present_when_next_started.lock().unwrap(),
        Some(false),
        "the next otkat starts after the unfinished file is gone"
    );
    assert!(!broken.exists());
    assert!(
        !next.exists(),
        "the next file is not written until its own turn"
    );

    gate.let_one_finish();
    first_write.join().unwrap();
    second_write.join().unwrap();

    assert!(!broken.exists());
    assert_eq!(fs::read(&next).unwrap(), b"45");
    assert_eq!(ring.later_plashka(), Some(PlashkaEvent::ClipDidNotSave));
    assert_eq!(
        ring.later_plashka(),
        None,
        "a finished otkat raises no alarm"
    );
}
