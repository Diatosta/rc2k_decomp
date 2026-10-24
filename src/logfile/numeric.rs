//! What the six numeric log functions share. Each does what `log_write` does, with one step
//! in between: under the log's lock it formats the number in eax as text in the scratch
//! buffer `dst` (0x93E030), ends it with a NUL, and goes on into `write_line` with eax = the
//! scratch buffer and edx as given.
//!
//! Registers: as `log_write`. If the tail's own level gate skips the line, eax is the
//! scratch buffer and ebx the byte after the text's NUL. With `g_logLevel` 0 only eax
//! changes, to the address after that function's call to the lock function.
//! Each of the six has four entries before it that set edx (`_l1`, `_l1_nl`, `_l2`,
//! `_l2_nl`) and jump to it, so they go through its hook.

use crate::errors::BadInput;
use crate::framework::{Regs, game_addr};
use crate::logfile::write_line::{acquire, enabled, level, write_line_regs};
use crate::moved::Place;

/// `dst` (0x93E030 in the image, 0x50 bytes below the log's line buffer): where a number's
/// text is made, and `start_log`'s date. Written under the log's lock.
pub const SCRATCH_LEN: usize = 0x50;
pub static SCRATCH: Place<[u8; SCRATCH_LEN]> = Place::new([0; SCRATCH_LEN]);
/// The longest text accepted: the scratch buffer with room for the NUL.
const SCRATCH_TEXT: usize = SCRATCH_LEN - 1;

pub fn scratch() -> *mut u8 {
    SCRATCH.ptr().cast()
}

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
    /// The address after its call to the lock function: what eax holds if `g_logLevel` is 0.
    pub after_lock_call_rva: usize,
    /// The text of the number (eax at entry) and the second value some take (ecx).
    pub text: fn(u32, u32) -> Result<NumberText, BadInput>,
}

/// The function with the original's registers in and out. Refuses a text that does not fit
/// the scratch buffer, before anything is done.
pub unsafe fn log_number_regs(body: &Body, r: &mut Regs) -> Result<(), BadInput> {
    unsafe {
        if enabled() <= 0 {
            return Ok(());
        }
        if level() == 0 {
            r.eax = game_addr(body.after_lock_call_rva) as u32;
            return Ok(());
        }
        let text = (body.text)(r.eax, r.ecx)?;
        if text.bytes.len() > SCRATCH_TEXT || text.nul_at > SCRATCH_TEXT {
            return Err(BadInput(
                "the number's text does not fit the scratch buffer",
            ));
        }
        acquire();
        std::ptr::copy_nonoverlapping(text.bytes.as_ptr(), scratch(), text.bytes.len());
        scratch().add(text.nul_at).write(0);
        r.eax = scratch() as u32;
        r.ebx = scratch() as u32 + text.nul_at as u32 + 1;
        write_line_regs(r);
    }
    Ok(())
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

/// `dispatch_number` without its hook (a refusal is the caller's).
pub fn number_entry(r: &mut Regs, body: &Body) -> Result<(), crate::errors::Refused> {
    unsafe { log_number_regs(body, r) }?;
    Ok(())
}

// ---- self-test: appends lines to the real var\logfile.txt ----
