//! `scene::cached_reflection_coordinates` (0x451908): `scene::reflection_coordinates`
//! behind a table of results kept.
//!
//! In:  nothing; the two numbers: the normal's and the point's (the object's reflection
//! cells)
//!
//! With the reflection cache switch at 0 the original jumps straight on into the working
//! out. Else the normal's number & 0x3FF is a place in four tables of 0x400 dwords (the
//! normal's number, the point's, and the two coordinates): if both numbers there are the ones
//! asked for, the coordinates come from the table; else they are worked out and all four
//! kept.
//! //! One caller, `scene::draw_object_face`.

use crate::errors::Refused;
use crate::scene::reflection_coordinates::reflection_coordinates;

// ---- game data ----
/// The four tables (`ObjectPlace.kept_reflections`), 0x400 dwords apart: the normal's
/// number, the point's, the two coordinates.
const TABLE_CELLS: usize = 0x400;

// ---- Rust version ----

// Converted: `rc2k_core::scene::reflection::{cache_place, cached}`
pub fn cached_reflection_coordinates(
    part: &crate::workers::PartScratch,
    reads: crate::workers::DrawReads<'_>,
    object: &crate::scene::object_place::ObjectPlace,
    m: &crate::scene::view_matrices::ViewMatrices,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), Refused> {
    use rc2k_core::scene::reflection::{cache_place, cached};
    if view.face.reflections_kept.get() == 0 {
        return Ok(reflection_coordinates(part, reads, object, m)?);
    }
    let asked: [u32; 2] = [
        object.reflection_normal_number.get(),
        object.reflection_point_number.get(),
    ];
    let place = cache_place(asked[0]);
    let slot = |table: usize| &object.kept_reflections[table * TABLE_CELLS + place];
    if let Some(coordinates) = cached(asked, [slot(0).get(), slot(1).get()], || {
        [slot(2).get(), slot(3).get()]
    }) {
        crate::session::write_bits(&object.reflection_coordinates, coordinates);
        return Ok(());
    }
    reflection_coordinates(part, reads, object, m)?;
    let coordinates: [u32; 2] = crate::session::read_bits(&object.reflection_coordinates);
    for (table, value) in [asked[0], asked[1], coordinates[0], coordinates[1]]
        .into_iter()
        .enumerate()
    {
        slot(table).set(value);
    }
    Ok(())
}
