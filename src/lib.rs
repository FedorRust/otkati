use std::path::PathBuf;

/// The key combination that asks the ring to save an otkat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bind {
    text: String,
}

impl Bind {
    /// A bind written the way a person sets it, for example `Alt+Z`.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// How the bind is written, for example `Alt+Z`.
    pub fn text(&self) -> &str {
        &self.text
    }
}

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
    /// The bind a new ring listens for, before anyone changes it.
    pub fn default_bind() -> Bind {
        Bind::new("Alt+Z")
    }

    /// `otkat_folder` is where a saved otkat would be written.
    pub fn new(otkat_folder: impl Into<PathBuf>) -> Self {
        Self {
            otkat_folder: otkat_folder.into(),
        }
    }

    /// `chord` reached the ring.
    ///
    /// When it is the ring's bind and the ring holds no frame, answers with
    /// [`PlashkaEvent::NothingToSave`] and does not create a file in the otkat
    /// folder. Any other chord does not save.
    pub fn bind(&mut self, chord: Bind) -> Option<PlashkaEvent> {
        if chord != Self::default_bind() {
            return None;
        }
        let _ = &self.otkat_folder;
        Some(PlashkaEvent::NothingToSave)
    }
}
