//! `scene::scale_texture_coordinates`: the polygon's texture coordinates from whole numbers to
//! floats.
//!
//! For each entry of the polygon, with the switch `SceneDraw.polygon_sides[0]` its first pair of
//! coordinates (`u`, `v`) and with `SceneDraw.polygon_sides[1]` its second (`u2`, `v2`): a value
//! that is not 0 becomes a float whose bits then lose 0x0C000000: the number over 2^24.

use crate::errors::Refused;
use crate::scene::polygon::{Listed, listed};

// ---- game data ----

// Logic: `rc2k_core::scene::texture_coordinates`
pub fn scale_texture_coordinates(
    polygon_work: &crate::scene::polygon_store::PolygonWork,
    poly: &crate::scene::polygon_work_store::PolygonWork,
) -> Result<(), Refused> {
    if (poly.polygon_vertex_count.get() as i32) <= 0 {
        return Ok(());
    }
    scale_entries(poly, &listed(polygon_work, poly)?)
}

/// `scale_texture_coordinates` on the polygon's entries as listed (a polygon of some).
pub fn scale_entries(
    poly: &crate::scene::polygon_work_store::PolygonWork,
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
