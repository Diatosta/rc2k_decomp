//! `scene::fog_at` (0x44E260): the fog's alpha at a depth: eax (0 when below 0) >> 13,
//! held below `g_fogTableLength` (the last entry past it), its byte of `g_fogTable` in the
//! top byte, flipped (`xor 0xFF000000`). The table and its length are the view's
//! (`scene::fog::Fog`).
//!
//! Takes: the depth (whole)
//! Returns: the alpha in its top byte
//!
//! Callers: the marks' drawers (16). The callers go on with a `mov`: no flags read. Refuses a
//! table length of 0 or past the table's 0x1000 bytes (the original reads past it).

use crate::errors::Refused;

/// On the view's fog.
pub fn fog_at(fog: &crate::scene::fog::Fog, depth: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::scene::fog::fog_at`.
    Ok(rc2k_core::scene::fog::fog_at(
        depth,
        fog.length.bits() as i32,
        |n| fog.level(n as u32),
    )?)
}
