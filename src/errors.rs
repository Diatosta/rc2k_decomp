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

/// Either reason, for functions that can meet both.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    Fatal(FatalError),
    BadInput(BadInput),
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
