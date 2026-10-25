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
    screen: Option<Screen>,
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
            screen: None,
        }
    }

    /// Enter a session.
    ///
    /// `remembered` is the screen kept from an earlier answer. Without one,
    /// the session is waiting for the person: a bind reports
    /// [`PlashkaEvent::NoAccess`] and writes no otkat. The next entry that
    /// receives that screen does not ask again, and the screen is the ring's
    /// monitor.
    pub fn enter_session(otkat_folder: impl Into<PathBuf>, remembered: Option<Screen>) -> Self {
        let waiting_for_screen = remembered.is_none();
        Self {
            otkat_folder: otkat_folder.into(),
            waiting_for_screen,
            screen: remembered,
        }
    }

    /// Whether this session still asks which screen to give the program.
    pub fn asks_which_screen(&self) -> bool {
        self.waiting_for_screen
    }

    /// The person answered with this screen. It is the ring's monitor, and
    /// the session stops asking.
    pub fn remember_screen(&mut self, screen: Screen) {
        self.screen = Some(screen);
        self.waiting_for_screen = false;
    }

    /// Offer a screen to the ring.
    ///
    /// Until the person has answered, the screen does not enter the ring
    /// and does not become the ring's monitor.
    pub fn offer_screen(&mut self, _screen: Screen) {}

    /// The ring's monitor: the screen from the person's answer.
    pub fn monitor(&self) -> Option<&Screen> {
        self.screen.as_ref()
    }

    /// Output sound arrived. It does not ask which screen to give.
    pub fn output_sound(&mut self) {}

    /// The microphone arrived. It does not ask which screen to give.
    pub fn microphone(&mut self) {}

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
