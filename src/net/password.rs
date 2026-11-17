//! The passwords of the network game. The host's, set on the network settings screen and kept in
//! `var/rc2k.toml`, locks the session a machine hosts (never in the log). A joiner's is asked
//! for when the session chosen is locked, answers the host's challenge and is kept in memory for
//! this join only. Also the reason the last join was refused, for the message the joiner is shown.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

static PASSWORD: Mutex<Vec<u8>> = Mutex::new(Vec::new());
static JOINING: Mutex<Vec<u8>> = Mutex::new(Vec::new());
static CANCELLED: AtomicBool = AtomicBool::new(false);
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

/// The password a joiner answers a locked session with (empty: none).
pub fn joining() -> Vec<u8> {
    JOINING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// Keeps the joiner's answer `text` (up to its first NUL).
pub fn set_joining(text: &[u8]) {
    let end = text.iter().position(|&b| b == 0).unwrap_or(text.len());
    *JOINING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = text[..end].to_vec();
}

/// Notes whether the joiner went back from the password question.
pub fn note_cancelled(cancelled: bool) {
    CANCELLED.store(cancelled, Ordering::SeqCst);
}

/// Whether the joiner went back from the password question.
pub fn cancelled() -> bool {
    CANCELLED.load(Ordering::SeqCst)
}
