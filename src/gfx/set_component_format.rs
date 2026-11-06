//! `gfx::set_component_format` (0x405DB4): describes the format of the game's own
//! component pixels.
//!
//! In:  nothing
//! Out: every register preserved
//! The format at `unk_4F3FAC` gets the masks 0xFF, 0xFF00, 0xFF0000 and 0xFF000000 and
//! is described by `gfx::describe_format`. One caller, `start`, which calls it directly (2026-10-23).

use crate::errors::Refused;
use crate::gfx::describe_format::describe_format;
use crate::gfx::formats::Formats;

// ---- game data ----

/// Converted: `rc2k_core::gfx::pixels::COMPONENT_MASKS`.
const MASKS: [u32; 4] = rc2k_core::gfx::pixels::COMPONENT_MASKS;

// ---- Rust version ----

pub fn set_component_format(formats: &Formats) -> Result<(), Refused> {
    let format = &formats.component_format;
    for (n, mask) in MASKS.into_iter().enumerate() {
        format[1 + n].set(mask);
    }
    describe_format(formats, format);
    Ok(())
}
