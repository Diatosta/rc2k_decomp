//! `scene::cached_reflection_coordinates` (0x451908): `scene::reflection_coordinates`
//! behind a table of results kept.
//!
//! In:  nothing; the two numbers in `dword_6399A8` and `dword_6399AC`
//!
//! With `dword_63D9B8` at 0 the original jumps straight on into the working out.
//! Else the normal's number & 0x3FF is a place in four tables of 0x400 dwords
//! (`dword_6399B8` the normal's number, `dword_63A9B8` the point's, `dword_63B9B8` and
//! `dword_63C9B8` the coordinates): if both numbers there are the ones asked for,
//! the coordinates come from the table; else they are worked out and all four kept.
//! One caller, 0x45188C.

use crate::errors::Refused;
use crate::scene::reflection_coordinates::reflection_coordinates;

// ---- game data ----
/// The four tables (`ObjectPlace.kept_reflections`), 0x400 dwords apart: the normal's
/// number, the point's, the two coordinates.
const TABLE_CELLS: usize = 0x400;

// ---- Rust version ----

// Converted: `rc2k_core::scene::reflection::{cache_place, cached}`
pub fn cached_reflection_coordinates(
    race: &crate::race::owner::Race,
    object: &crate::scene::object_place::ObjectPlace,
    m: &crate::scene::view_matrices::ViewMatrices,
) -> Result<(), Refused> {
    use rc2k_core::scene::reflection::{cache_place, cached};
    if race.shared.view.reflections_kept.bits() == 0 {
        return Ok(reflection_coordinates(race, object, m)?);
    }
    let asked: [u32; 2] = [
        object.reflection_normal_number.bits(),
        object.reflection_point_number.bits(),
    ];
    let place = cache_place(asked[0]);
    let slot = |table: usize| &object.kept_reflections[table * TABLE_CELLS + place];
    if let Some(coordinates) = cached(asked, [slot(0).bits(), slot(1).bits()], || {
        [slot(2).bits(), slot(3).bits()]
    }) {
        crate::session::write_bits(&object.reflection_coordinates, coordinates);
        return Ok(());
    }
    reflection_coordinates(race, object, m)?;
    let coordinates: [u32; 2] = crate::session::read_bits(&object.reflection_coordinates);
    for (table, value) in [asked[0], asked[1], coordinates[0], coordinates[1]]
        .into_iter()
        .enumerate()
    {
        slot(table).set_bits(value);
    }
    Ok(())
}
