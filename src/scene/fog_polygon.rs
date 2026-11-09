//! `scene::fog_polygon` (0x44E286): each of the polygon's points gets its fog by its
//! depth.
//!
//! For each entry: the depth of its point's record (+0x18) as a whole number (`fistp`),
//! under 0 taken as 0, over 0x2000 is the place in the table
//! `byte_63661C` (0x44E260; its length `dword_63761C`, a place past it is the last
//! one); the entry's byte +0x20 is that table's byte inverted. With `dword_6365C8` at 4
//! the byte is the low byte of `dword_636610` inverted instead. The table, its length and
//! the next step are the view's (`scene::fog::Fog`).
//! One caller, the six-byte 0x44E25A, which six functions call.
//! Refuses a polygon of no vertices (the original goes through 2^32), an entry without
//! its record and a table's length above 0x1000. A length of 0, as the menus have it,
//! makes every place the byte before the table (the original's `length - 1`), which nothing
//! writes: 0.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::scene::polygon::{Listed, listed};

// ---- game data ----
const TABLE_LEN: u32 = crate::scene::fog::TABLE_LEN as u32;

// ---- Rust version ----

pub fn fog_polygon(
    scene: &crate::scene::owner::Scene,
    fog: &crate::scene::fog::Fog,
) -> Result<(), Refused> {
    fog_entries(&listed()?, scene, fog)
}

/// `fog_polygon` on the polygon's entries as listed.
pub fn fog_entries(
    entries: &Listed,
    scene: &crate::scene::owner::Scene,
    fog: &crate::scene::fog::Fog,
) -> Result<(), Refused> {
    if entries.is_empty() {
        return Err(BadInput("a polygon of no vertices").into());
    }
    if entries.iter().any(|entry| entry.record.get() == 0) {
        return Err(BadInput("an entry without its record").into());
    }
    let length: u32 = fog.length.bits();
    if length > TABLE_LEN {
        return Err(BadInput("a fog table longer than its room").into());
    }
    let fixed = (scene.lights.state.view_kind.bits() == 4).then(|| fog.next.bits());
    for entry in entries {
        let record = entry.record.get();
        let depth = x87::to_int(crate::scene::point_records::vertex(record, 0x18));
        // A place past the length is the last one; with no length (the menus)
        // that is the byte before the table, as in the original.
        let place = depth.max(0) as u32 >> 13;
        let place = if (place.wrapping_sub(length) as i32) < 0 {
            place
        } else {
            length.wrapping_sub(1)
        };
        let level = u32::from(fog.level(place));
        entry.fog.set((fixed.unwrap_or(level) ^ 0xFF) as u8);
    }
    Ok(())
}
