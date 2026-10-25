use std::path::PathBuf;

use otkati::SessionAutostart;

fn ring_program_dir() -> PathBuf {
    let test_exe = std::env::current_exe().expect("test executable");
    test_exe
        .parent()
        .and_then(|deps| deps.parent())
        .expect("the ring program lives beside the test executable")
        .to_path_buf()
}

#[test]
fn after_hyprland_login_the_ring_is_already_running() {
    let mut session = SessionAutostart::shipped().log_in(ring_program_dir());

    assert!(
        session.ring_is_running(),
        "after login the ring is already running, before anyone opens it"
    );
}
