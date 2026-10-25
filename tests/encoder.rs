use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use otkati::{BindDay, Card, Encoder, PlashkaEvent, Ring};

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
        let path = std::env::temp_dir().join(format!("otkati-encoder-{tick}-{n}"));
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
fn a_new_machine_offers_h264_and_chooses_it() {
    let folder = OtkatFolder::new();
    let ring = Ring::new(folder.path.clone());

    assert_eq!(ring.encoder(), Encoder::H264);
    assert_eq!(ring.offered_encoders(), vec![Encoder::H264]);

    let session = Ring::enter_session(folder.path.clone(), None);
    assert_eq!(session.encoder(), Encoder::H264);
    assert_eq!(session.offered_encoders(), vec![Encoder::H264]);
}

#[test]
fn a_3060_ti_offers_h264_and_hevc_and_still_chooses_h264() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::new(folder.path.clone());

    ring.see_card(Card::Nvidia3060Ti);

    assert_eq!(ring.encoder(), Encoder::H264);
    assert_eq!(
        ring.offered_encoders(),
        vec![Encoder::H264, Encoder::Hevc]
    );
}

#[test]
fn switching_to_hevc_on_a_3060_ti_keeps_the_mp4_name() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.see_card(Card::Nvidia3060Ti);
    ring.hold(Duration::from_secs(1));

    ring.take_encoder(Encoder::Hevc);

    assert_eq!(ring.encoder(), Encoder::Hevc);
    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.name(), "otkat1-2026-10-25.mp4");
    drop(event);
    assert!(folder.path.join("otkat1-2026-10-25.mp4").is_file());
}

#[test]
fn choosing_av1_on_a_3060_ti_leaves_the_current_encoder() {
    let folder = OtkatFolder::new();
    let mut ring = Ring::with_clock(folder.path.clone(), || BindDay::new(2026, 10, 25));
    ring.see_card(Card::Nvidia3060Ti);
    ring.take_encoder(Encoder::Hevc);
    ring.hold(Duration::from_secs(1));

    ring.take_encoder(Encoder::Av1);

    assert_eq!(ring.encoder(), Encoder::Hevc);
    let event = ring.bind(Ring::default_bind());
    let Some(PlashkaEvent::Saved(saved)) = &event else {
        panic!("expected a saved otkat, got {event:?}");
    };
    assert_eq!(saved.name(), "otkat1-2026-10-25.mp4");
}
