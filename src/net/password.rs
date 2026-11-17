//! The session's password as typed on the network settings screen: kept for this run only, in
//! memory (never in a file, never in the log). A host locks its session with it; a joiner
//! answers a locked host's challenge with it. Also the reason the last join was refused, for the
//! message the joiner is shown.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

static PASSWORD: Mutex<Vec<u8>> = Mutex::new(Vec::new());
static REFUSED: AtomicBool = AtomicBool::new(false);

/// The password typed (empty: none).
pub fn get() -> Vec<u8> {
    PASSWORD
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// Keeps the password `text` (up to its first NUL).
pub fn set(text: &[u8]) {
    let end = text.iter().position(|&b| b == 0).unwrap_or(text.len());
    *PASSWORD
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = text[..end].to_vec();
}

/// What the settings screen shows for a password that is not being edited: a star a character.
pub fn masked() -> Vec<u8> {
    vec![b'*'; get().len()]
}

/// Notes whether the last join was refused for its password.
pub fn note_refused(refused: bool) {
    REFUSED.store(refused, Ordering::SeqCst);
}

/// Whether the last join was refused for its password.
pub fn refused() -> bool {
    REFUSED.load(Ordering::SeqCst)
}
