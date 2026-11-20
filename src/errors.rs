//! How the Rust code says "not here". A fatal error or a refused input ends the program with
//! its report (`fatal::raise::refused`).

/// A fatal error of the game: `code` is the number the report shows and `message` the text with
/// it. It is raised through `fatal::raise`, which ends the game with the report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FatalError {
    pub code: u32,
    /// Empty where the report has no text.
    pub message: &'static str,
}

/// An input the Rust code refuses: the game itself goes wrong there (a division that traps, a
/// write past a buffer, a loop that never ends) or its behaviour there has not been worked out.
/// The program ends with the reason.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BadInput(pub &'static str);

/// A trouble the game can go back from: a file of the player's or a stage's data that cannot
/// be used. The reason is a short text for the player. Where a menu or a race is running it
/// ends in the main menu (`error_dialog::recover`), after the player has been told.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recover(pub &'static str);

/// Either reason, for functions that can meet them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    Fatal(FatalError),
    BadInput(BadInput),
    Recoverable(Recover),
}

impl From<Recover> for Refused {
    fn from(e: Recover) -> Self {
        Refused::Recoverable(e)
    }
}

impl From<FatalError> for Refused {
    fn from(e: FatalError) -> Self {
        Refused::Fatal(e)
    }
}

impl From<BadInput> for Refused {
    fn from(e: BadInput) -> Self {
        Refused::BadInput(e)
    }
}

impl From<rc2k_core::Invalid> for BadInput {
    fn from(e: rc2k_core::Invalid) -> Self {
        BadInput(e.0)
    }
}

impl From<rc2k_core::Invalid> for Refused {
    fn from(e: rc2k_core::Invalid) -> Self {
        Refused::BadInput(e.into())
    }
}

/// The carry and zero results a function returns for callers that test them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct CarryZeroFlags {
    pub carry: bool,
    pub zero: bool,
}

/// Reads for closures that a core function calls (they cannot return an error): the first
/// refusal is kept and the reads after it answer 0; the caller runs `check` right after the
/// core call, before it uses anything the core made.
#[derive(Default)]
pub(super) struct Fault(std::cell::Cell<Option<BadInput>>);

impl Fault {
    /// The value, or the type's default after keeping the refusal.
    pub fn ok<T: Default>(&self, r: Result<T, BadInput>) -> T {
        r.unwrap_or_else(|e| {
            if self.0.get().is_none() {
                self.0.set(Some(e));
            }
            T::default()
        })
    }

    pub fn check(&self) -> Result<(), BadInput> {
        self.0.take().map_or(Ok(()), Err)
    }
}
