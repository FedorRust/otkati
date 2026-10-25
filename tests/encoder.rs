use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use otkati::{Encoder, Ring};

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
