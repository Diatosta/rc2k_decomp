//! `gfx::set_component_format` (0x405DB4): describes the format of the game's own
//! component pixels.
//!
//! In:  nothing
//! Out: every register preserved
//! The format at `unk_4F3FAC` gets the masks 0xFF, 0xFF00, 0xFF0000 and 0xFF000000 and
//! is described by `gfx::describe_format`. One caller, `start`, which calls it directly (2026-10-23).

use crate::errors::Refused;
use crate::gfx::describe_format::describe_format;
use crate::gfx::formats::FORMATS;

// ---- game data ----

/// Converted: `rc2k_core::gfx::pixels::COMPONENT_MASKS`.
const MASKS: [u32; 4] = rc2k_core::gfx::pixels::COMPONENT_MASKS;

// ---- Rust version ----

pub unsafe fn set_component_format() -> Result<(), Refused> {
    unsafe {
        let format = FORMATS.component_format.as_ptr() as *mut u32;
        for (n, mask) in MASKS.into_iter().enumerate() {
            format.add(1 + n).write_unaligned(mask);
        }
        describe_format(format)?;
    }
    Ok(())
}
