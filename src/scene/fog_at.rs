//! `scene::fog_at`: the fog's alpha at a depth: the depth (0 when below 0) >> 13, held below
//! the fog table's length (the last entry past it), its byte of the fog table in the top byte,
//! flipped (`xor 0xFF000000`). The table and its length are the view's (`scene::fog::Fog`).
//! Takes the depth (whole) and returns the alpha in its top byte.
//!
//! Refuses a table length of 0 or past the table's 0x1000 bytes.

use crate::errors::Refused;

/// On the view's fog.
pub fn fog_at(fog: &crate::scene::fog::Fog, depth: u32) -> Result<u32, Refused> {
    // Logic: `rc2k_core::scene::fog::fog_at`.
    Ok(rc2k_core::scene::fog::fog_at(
        depth,
        fog.length.get() as i32,
        |n| fog.level(n as u32),
    )?)
}
