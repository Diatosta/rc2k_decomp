//! `scene::scale_texture_coordinates` (0x44E2FB): the polygon's texture coordinates
//! from whole numbers to floats.
//!
//! For each entry of the polygon, with the switch `SceneDraw.polygon_sides[0]` its first pair of
//! coordinates (+8, +0xC) and with `SceneDraw.polygon_sides[1]` its second (+0x10, +0x14): a value
//! that is not 0 becomes a float (`fild`, `fstp`) whose bits then lose 0x0C000000: the
//! number over 2^24. (The original also leaves the last value and its float in the work
//! variables `Machine.work` and `Machine.work_float`, which nothing reads: not kept, so that threads
//! drawing at once share no write.)
//! Eight callers.

use crate::errors::Refused;
use crate::scene::polygon::{Listed, listed};

// ---- game data ----

// ---- Rust version ----

// Converted: `rc2k_core::scene::texture_coordinates`
pub fn scale_texture_coordinates(
    polygon_work: &'static crate::scene::polygon_store::PolygonWork,
    poly: &crate::scene::polygon_work::PolygonWork,
) -> Result<(), Refused> {
    if (poly.polygon_vertex_count.get() as i32) <= 0 {
        return Ok(());
    }
    scale_entries(poly, &listed(polygon_work, poly)?)
}

/// `scale_texture_coordinates` on the polygon's entries as listed (a polygon of some).
pub fn scale_entries(
    poly: &crate::scene::polygon_work::PolygonWork,
    entries: &Listed,
) -> Result<(), Refused> {
    use rc2k_core::scene::texture_coordinates as coordinates;
    let switches: [u32; 2] = [poly.polygon_sides[0].get(), poly.polygon_sides[1].get()];
    for entry in entries {
        for (pair, switch) in switches.into_iter().enumerate() {
            if switch == 0 {
                continue;
            }
            for at in [8 + pair * 8, 0xC + pair * 8] {
                let value = entry.word(at);
                entry.set_word(at, coordinates::scaled(value));
            }
        }
    }
    Ok(())
}
