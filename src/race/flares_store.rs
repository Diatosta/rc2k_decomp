//! The flares' tables (IDA 0x63F58C .. 0x64098C; phase 7, typed): five of the six flare
//! tables (the first, the places' x, is `LEVEL_GRID.flare_place_x`, 0x63F18C): the places'
//! y and z, the other light, the size and the object number, each 0x400 bytes. The tables
//! hold 0x100 flares but `race::frame_draw::add_flare` runs on to 0x400 (a slip of the
//! original's: it writes into the tables that follow, so its code reaches them by address).
//!
//! Names are readings unless IDA's.
//! The race's own copy is unused: each view draws its flares in its own
//! (`race::lamp_work::LampWork::flares`).

use crate::cells::ZeroCells;
use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct FlareTables {
    /// 0x63F58C: `dword_63F58C`: the flares' y (`race::frame_draw::add_flare`, `draw_flare`).
    pub place_y: [Dword; 256],
    /// 0x63F98C: `dword_63F98C`: the flares' z.
    pub place_z: [Dword; 256],
    /// 0x63FD8C: `dword_63FD8C`: the flares' colour (`g_otherLight`).
    pub other_light: [Dword; 256],
    /// 0x64018C: `dword_64018C`: the flares' size.
    pub size: [Dword; 256],
    /// 0x64058C: `dword_64058C`: the flares' object number.
    pub object_number: [Dword; 256],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece (a field of `race::owner::Race::course`), with the image's first values.
impl FlareTables {
    pub const fn new() -> Self {
        ZeroCells::ZERO
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            crate::race::owner::RACE.course.flare_tables(),
        );
    }
}
