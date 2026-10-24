//! `gfx::clear_picture` (0x4060D1): fills the picture buffer with zeros.
//!
//! In:  nothing
//! Out: every register preserved
//! 0x75300 dwords (800 x 600) at the buffer `dword_4F49DC` points at, whatever size the
//! picture in it has. Three callers, the picture loader `sub_411863` among them.
//! Refuses a null buffer pointer. The name is a reading.

use crate::errors::BadInput;
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
// without running the original. The Rust version writes zeros.
