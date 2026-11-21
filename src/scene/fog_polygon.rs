//! `scene::fog_polygon`: each of the polygon's points gets its fog by its depth.
//!
//! For each entry: the depth of its point's record (its depth) as a whole number, under 0 taken as
//! 0, over 0x2000 is the place in the view's fog table (`scene::fog_at`; a place past the table's
//! length is the last one); the entry's fog byte is that table's byte inverted. With the view kind
//! at 4 the byte is the low byte of the next step inverted instead. The table, its length and the
//! next step are the view's (`scene::fog::Fog`). Refuses a polygon of no vertices, an entry without
//! its record and a table's length above 0x1000. A length of 0, as the menus have it, makes every
//! place the byte before the table, which nothing writes: 0.

use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::scene::polygon::{Listed, listed};

// ---- game data ----
const TABLE_LEN: u32 = crate::scene::fog::TABLE_LEN as u32;

pub fn fog_polygon(
    part: &crate::scene::scratch_store::PartScratch,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), Refused> {
    let polygon_work = &part.polygon;
    fog_entries(part, &listed(polygon_work, &view.polygon)?, view)
}

/// `fog_polygon` on the polygon's entries as listed.
pub fn fog_entries(
    part: &crate::scene::scratch_store::PartScratch,
    entries: &Listed,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), Refused> {
    let (fog, light) = (&view.fog, &view.light);
    if entries.is_empty() {
        return Err(BadInput("a polygon of no vertices").into());
    }
    if entries.iter().any(|entry| entry.record.get() == 0) {
        return Err(BadInput("an entry without its record").into());
    }
    let length = table_length(view)?;
    let fixed = (light.view_kind.get() == 4).then(|| fog.borrow().next);
    for entry in entries {
        entry
            .fog
            .set(byte(part, view, entry.record.get(), length, fixed));
    }
    Ok(())
}

/// The view's fog table's length; refused above its room.
fn table_length(view: &crate::race::view_scratch::ViewScratch) -> Result<u32, BadInput> {
    let length: u32 = view.fog.borrow().length;
    if length > TABLE_LEN {
        return Err(BadInput("a fog table longer than its room"));
    }
    Ok(length)
}

/// The fog byte of the point whose record is `record`, as `fog_entries` gives an entry: by
/// the record's depth (`length` the table's), or `fixed`'s low byte, inverted.
fn byte(
    part: &crate::scene::scratch_store::PartScratch,
    view: &crate::race::view_scratch::ViewScratch,
    record: u32,
    length: u32,
    fixed: Option<u32>,
) -> u8 {
    let depth = x87::to_int(crate::scene::point_records::vertex(
        part,
        &view.polygon.quad_vertices,
        record,
        0x18,
    ));
    // A place past the length is the last one; with no length (the menus) that is the byte
    // before the table, which is 0.
    let place = depth.max(0) as u32 >> 13;
    let place = if (place.wrapping_sub(length) as i32) < 0 {
        place
    } else {
        length.wrapping_sub(1)
    };
    let level = u32::from(view.fog.borrow().level(place));
    (fixed.unwrap_or(level) ^ 0xFF) as u8
}

/// The fog byte `fog_entries` will give an entry whose record is `record`, outside the view
/// kind 4 (whose byte is fixed): what a reflective object face's shine is scaled by
/// (`scene::draw_object_face`). Refuses as `fog_entries` does a table longer than its room.
pub fn fog_of_record(
    part: &crate::scene::scratch_store::PartScratch,
    view: &crate::race::view_scratch::ViewScratch,
    record: u32,
) -> Result<u8, BadInput> {
    Ok(byte(part, view, record, table_length(view)?, None))
}
