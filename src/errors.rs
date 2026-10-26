//! How a Rust version says "not here". A fatal error is raised from Rust
//! (`fatal::raise::refused`); a refused input is handed back to the original code, and a
//! build that runs without the game needs an answer for every one of those.

/// A condition the original reports through its fatal-error reporter at 0x40763C: it stores
/// `code` in 0x4F52A4 and a pointer to `message` (or null) in 0x4F52A8, then jumps there.
/// A hook raises it through `fatal::raise`, which ends the game with the report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FatalError {
    pub code: u32,
    /// Empty where the original stores a null pointer.
    pub message: &'static str,
}

/// An input the Rust version refuses: the original itself goes wrong there (a division that
/// traps, a write past a buffer, a loop that never ends) or its behaviour there has not been
/// worked out. A hook hands it to the original, so the game behaves as it always did.
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
