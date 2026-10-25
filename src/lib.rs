use std::path::PathBuf;

/// What the ring tells the face to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlashkaEvent {
    /// The ring holds no frame, so there is nothing to save.
    NothingToSave,
}

/// Encoded seconds the ring holds, and the bind that asks to save an otkat.
///
/// A newly created ring holds no frame.
pub struct Ring {
    otkat_folder: PathBuf,
}

impl Ring {
    /// `otkat_folder` is where a saved otkat would be written.
    pub fn new(otkat_folder: impl Into<PathBuf>) -> Self {
        Self {
            otkat_folder: otkat_folder.into(),
        }
    }

    /// The bind was pressed.
    ///
    /// An empty ring answers with [`PlashkaEvent::NothingToSave`] and does not
    /// create a file in the otkat folder.
    pub fn bind(&mut self) -> PlashkaEvent {
        let _ = &self.otkat_folder;
        PlashkaEvent::NothingToSave
    }
}
