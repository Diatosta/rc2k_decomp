//! `scene::fog_at` (0x44E260): the fog's alpha at a depth: eax (0 when below 0) >> 13,
//! held below `g_fogTableLength` (the last entry past it), its byte of `g_fogTable` in the
//! top byte, flipped (`xor 0xFF000000`).
//!
//! In:  eax the depth (whole)
//! Out: eax the alpha in its top byte; the rest kept
//!
//! Callers: the marks' drawers (16). The callers go on with a `mov`: no flags read. Refuses a
//! table length of 0 or past the table's 0x1000 bytes (the original reads past it).

use crate::errors::Refused;
use crate::scene::light_store::LIGHT_STATE;

/// The Rust version (see the header).
pub fn fog_at(depth: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::scene::fog::fog_at`.
    let reading = crate::scene::reading::Reading::new();
    let answer = rc2k_core::scene::fog::fog_at(
        depth,
        LIGHT_STATE.fog_table_length.bits() as i32,
        // A byte of the table (the store types it in dwords).
        |n| {
            let dword = reading.keep(crate::session::read_on_in(
                &LIGHT_STATE,
                &LIGHT_STATE.fog_table,
                n / 4,
                "the fog table",
            ));
            (dword >> (8 * (n % 4))) as u8
        },
    );
    reading.finish()?;
    Ok(answer?)
}
