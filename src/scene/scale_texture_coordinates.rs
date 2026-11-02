//! `scene::scale_texture_coordinates` (0x44E2FB): the polygon's texture coordinates
//! from whole numbers to floats.
//!
//! In:  nothing
//! Out: eax = the last value worked on, ecx = the number of vertices, esi = the last
//!      entry; nothing changed for a polygon of no vertices; everything else preserved
//!
//! For each entry of the polygon, with the switch `dword_6240E0` its first pair of
//! coordinates (+8, +0xC) and with `dword_6240E4` its second (+0x10, +0x14): a value
//! that is not 0 becomes a float (`fild`, `fstp`) whose bits then lose 0x0C000000: the
//! number over 2^24. The two work variables `dword_5183C0` and `flt_5183C4` keep the
//! last value and its float.
//! Eight callers.

use crate::errors::Refused;
use crate::scene::draw_store::DRAW;
use crate::scene::polygon::{Listed, listed};
use crate::session::MACHINE;

// ---- game data ----

// ---- Rust version ----

// Converted: `rc2k_core::scene::texture_coordinates`
pub fn scale_texture_coordinates() -> Result<(), Refused> {
    if (DRAW.polygon_vertex_count.bits() as i32) <= 0 {
        return Ok(());
    }
    scale_entries(&listed()?)
}

/// `scale_texture_coordinates` on the polygon's entries as listed (a polygon of some).
pub fn scale_entries(entries: &Listed) -> Result<(), Refused> {
    use rc2k_core::scene::texture_coordinates as coordinates;
    let switches: [u32; 2] = [DRAW.polygon_sides[0].bits(), DRAW.polygon_sides[1].bits()];
    for entry in entries {
        for (pair, switch) in switches.into_iter().enumerate() {
            if switch == 0 {
                continue;
            }
            for at in [8 + pair * 8, 0xC + pair * 8] {
                let value = entry.word(at);
                if let Some(float) = coordinates::float_of(value) {
                    MACHINE.work.set(value);
                    MACHINE.work_float.set(float);
                }
                entry.set_word(at, coordinates::scaled(value));
            }
        }
    }
    Ok(())
}
