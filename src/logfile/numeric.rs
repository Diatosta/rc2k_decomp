//! What the six numeric log functions share. Each does what `log_write` does, with one step
//! in between: under the log's lock it formats the number as text in a scratch
//! buffer, ends it with a NUL, and goes on into `write_line` with that buffer and the
//! line-break flag and level as given.
//!
//! Each of the six has four entries before it that set the flag and level (`_l1`, `_l1_nl`,
//! `_l2`, `_l2_nl`) and jump to it, so they go through it.

use crate::errors::BadInput;

/// `dst` (0x93E030 in the image, 0x50 bytes below the log's line buffer): where a number's
/// text is made, and `start_log`'s date. Written under the log's lock.
pub const SCRATCH_LEN: usize = 0x50;

/// The text of a number, as it is left in the scratch buffer.
pub struct NumberText {
    /// The bytes written, from the start of the buffer.
    pub bytes: Vec<u8>,
    /// Where the NUL goes. `log_float` puts it where the wanted number of decimals ends,
    /// which can be before or after the end of `bytes`.
    pub nul_at: usize,
}

impl NumberText {
    pub fn plain(bytes: Vec<u8>) -> Self {
        let nul_at = bytes.len();
        Self { bytes, nul_at }
    }
}

/// One of the six functions.
pub struct Body {
    /// The text of the number and the second value some take (the digits or the flags).
    pub text: fn(u32, u32) -> Result<NumberText, BadInput>,
}

/// For Rust callers: the number's text logged (`write_line::log_text`), not through the
/// scratch buffer. The text ends where the original puts its NUL, but never past what was
/// written: the original logs the scratch buffer's leftovers there (`log_float` with more
/// decimals than its text has; `notes/bugs.md`, fixed here).
pub fn log_number(body: &Body, eax: u32, ecx: u32, newline: bool, level: u8) {
    // A text the scratch buffer cannot hold the original refuses (not logged): none of the
    // six makes one from a number.
    let Ok(text) = (body.text)(eax, ecx) else {
        return;
    };
    let end = text.nul_at.min(text.bytes.len());
    crate::logfile::write_line::log_text(&text.bytes[..end], newline, level);
}

// ---- self-test: appends lines to the real var\logfile.txt ----
