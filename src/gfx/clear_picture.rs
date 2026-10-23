//! `gfx::clear_picture` (0x4060D1): fills the picture buffer with zeros.
//!
//! In:  nothing
//! Out: every register preserved
//! 0x75300 dwords (800 x 600) at the buffer `dword_4F49DC` points at, whatever size the
//! picture in it has. Three callers, the picture loader `sub_411863` among them.
//! Refuses a null buffer pointer. The name is a reading.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::Regs;
use crate::gfx::picture::{PICTURE_DWORDS, picture};

// ---- Rust version ----

pub unsafe fn clear_picture() -> Result<(), BadInput> {
    let buffer = picture();
    if buffer.is_null() {
        return Err(BadInput("no picture buffer"));
    }
    unsafe { std::ptr::write_bytes(buffer, 0, PICTURE_DWORDS * 4) };
    Ok(())
}

// Written out: the buffer is nearly two megabytes, and what the function leaves is known
// without running the Rust version beside it. In shadow mode the original runs and the
// buffer is checked for what the Rust version writes, zeros.
/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    if let Err(e) = unsafe { clear_picture() } {
        return Err(e.into());
    }
    Ok(())
}
