//! `gfx::clear_picture` (0x4060D1): fills the picture buffer with zeros.
//!
//! 0x75300 dwords (800 x 600) at the buffer `dword_4F49DC` points at, whatever size the
//! picture in it has. Three callers, the picture loader `gfx::load_picture` among them.
//! Refuses a null buffer pointer. The name is a reading.

use crate::errors::BadInput;
use crate::gfx::picture::{PICTURE_DWORDS, has_picture, with_picture};

pub fn clear_picture() -> Result<(), BadInput> {
    if !has_picture() {
        return Err(BadInput("no picture buffer"));
    }
    with_picture(|b| b.get_mut(..PICTURE_DWORDS * 4).map(|p| p.fill(0)))?
        .ok_or(BadInput("a picture buffer shorter than a screen"))
}

// Written out: the buffer is nearly two megabytes, and what the function leaves is known
// without running the original. It writes zeros.
