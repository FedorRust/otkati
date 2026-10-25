use std::path::PathBuf;

/// The key combination that asks the ring to save an otkat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bind {
    text: &'static str,
}

impl Bind {
    /// How the bind is written, for example `Alt+Z`.
    pub fn text(self) -> &'static str {
        self.text
    }
}

/// What the ring tells the face to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlashkaEvent {
    /// The ring holds no frame, so there is nothing to save.
    NothingToSave,
    /// The person has not answered which screen the program may see.
    NoAccess,
}

/// A screen the person can give the program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Screen {
    name: String,
}

impl Screen {
    /// The screen the person chose.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// How this screen is told apart from another.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Encoded seconds the ring holds, and the bind that asks to save an otkat.
///
/// A newly created ring holds no frame. A session entered with no remembered
/// screen is still waiting for the person's answer.
pub struct Ring {
    otkat_folder: PathBuf,
    waiting_for_screen: bool,
}

impl Ring {
    /// The bind a new ring listens for, before anyone changes it.
    pub fn default_bind() -> Bind {
        Bind { text: "Alt+Z" }
    }

    /// `otkat_folder` is where a saved otkat would be written.
    pub fn new(otkat_folder: impl Into<PathBuf>) -> Self {
        Self {
            otkat_folder: otkat_folder.into(),
            waiting_for_screen: false,
        }
    }

    /// Enter a session.
    ///
    /// `remembered` is the screen kept from an earlier answer. Without one,
    /// the session is waiting for the person: a bind reports
    /// [`PlashkaEvent::NoAccess`] and writes no otkat.
    pub fn enter_session(otkat_folder: impl Into<PathBuf>, remembered: Option<Screen>) -> Self {
        Self {
            otkat_folder: otkat_folder.into(),
            waiting_for_screen: remembered.is_none(),
        }
    }

    /// The bind was pressed.
    ///
    /// An empty ring answers with [`PlashkaEvent::NothingToSave`] and does not
    /// create a file in the otkat folder. A session still waiting for the
    /// person's screen answers with [`PlashkaEvent::NoAccess`] and does not
    /// create a file either.
    pub fn bind(&mut self) -> PlashkaEvent {
        let _ = &self.otkat_folder;
        if self.waiting_for_screen {
            return PlashkaEvent::NoAccess;
        }
        PlashkaEvent::NothingToSave
    }
}
