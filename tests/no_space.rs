use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use otkati::{BindDay, PlashkaEvent, Ring};

/// Runs the body on a tmpfs with no free blocks.
///
/// The mount is visible only inside a fresh user and mount namespace, so the
/// check re-executes this test there.
fn enter_full_disk(test_name: &str) -> bool {
    if std::env::var_os("OTKATI_FULL_DISK").is_some() {
        return true;
    }
    let exe = std::env::current_exe().expect("test executable");
    let output = Command::new("unshare")
        .args(["--mount", "--user", "--map-root-user", "--"])
        .arg(&exe)
        .arg(test_name)
        .arg("--exact")
        .arg("--test-threads=1")
        .env("OTKATI_FULL_DISK", "1")
        .output()
        .unwrap_or_else(|err| panic!("could not enter a full disk: {err}"));
    assert!(
        output.status.success(),
        "full disk child failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    false
}

struct FullDisk {
    path: PathBuf,
}

impl FullDisk {
    fn new() -> Self {
        let tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("otkati-no-space-{tick}"));
        fs::create_dir_all(&path).unwrap();
        let status = Command::new("mount")
            .args(["-t", "tmpfs", "-o", "size=64k", "tmpfs"])
            .arg(&path)
            .status()
            .expect("mount");
        assert!(status.success(), "tmpfs did not mount");
        let mut filler = OpenOptions::new()
            .create(true)
            .write(true)
            .open(path.join("filler"))
            .unwrap();
        let block = vec![0_u8; 4096];
        loop {
            match filler.write(&block) {
                Ok(0) => break,
                Ok(n) if n < block.len() => break,
                Ok(_) => {}
                Err(err) if err.kind() == ErrorKind::StorageFull => break,
                Err(err) => panic!("filling the disk failed: {err}"),
            }
        }
        drop(filler);
        Self { path }
    }
}

impl Drop for FullDisk {
    fn drop(&mut self) {
        let _ = Command::new("umount").arg(&self.path).status();
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn mp4_duration_secs(path: &Path) -> u64 {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=nw=1:nk=1")
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ffprobe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    let secs: f64 = text.trim().parse().unwrap();
    secs.round() as u64
}

fn mp4_names(folder: &Path) -> Vec<String> {
    let mut names: Vec<_> = fs::read_dir(folder)
        .unwrap()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".mp4"))
        .collect();
    names.sort();
    names
}

#[test]
fn bind_when_the_disk_has_no_space_leaves_no_finished_otkat() {
    if !enter_full_disk("bind_when_the_disk_has_no_space_leaves_no_finished_otkat") {
        return;
    }
    let disk = FullDisk::new();
    let mut ring = Ring::with_clock(&disk.path, || BindDay::new(2026, 10, 25));
    ring.hold(Duration::from_secs(120));

    let event = ring.bind(Ring::default_bind());

    assert_eq!(event, Some(PlashkaEvent::NoSpace));
    drop(event);
    assert_eq!(
        mp4_names(&disk.path),
        Vec::<String>::new(),
        "a partial file must not remain as a finished otkat"
    );

    fs::remove_file(disk.path.join("filler")).unwrap();
    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected the first finished otkat once space returned, got {event:?}");
    };
    assert_eq!(saved.name(), "otkat1-2026-10-25.mp4");
    drop(event);

    assert_eq!(
        mp4_duration_secs(&disk.path.join("otkat1-2026-10-25.mp4")),
        120
    );
    assert!(!disk.path.join("otkat2-2026-10-25.mp4").exists());
}
