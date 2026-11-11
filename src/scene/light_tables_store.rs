//! The lighting tables (IDA 0x635118 .. 0x636588): the colours the lights
//! pick from (the light table, two lamp tables, the car light table, a third lamp table),
//! the view window's mode and the words around it. `#[repr(C)]` with the image's layout,
//! over the race block's bytes (`scene::owner::Scene::light_tables`). Names are readings unless IDA's.

use crate::session::{Bytes, Dword, Word};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct LightTables {
    /// +0x0 (0x635118): `g_lightTable`: 256 colours (`scene::vertex_light`, `physics::car_lighting`, `race::course_faces`).
    pub light_table: [Dword; 256],
    /// +0x400 (0x635518): `dword_635518`: lamp A's 256 colours.
    pub lamp_table_a: [Dword; 256],
    /// +0x800 (0x635918): `dword_635918`: lamp B's 256 colours.
    pub lamp_table_b: [Dword; 256],
    /// +0xc00 (0x635D18): `g_carLightTable`: 256 colours (`scene::add_car_light`, `race::frame_draw`).
    pub car_light_table: [Dword; 256],
    /// +0x1000 (0x636118): `unk_636118`: a third lamp table of 256 colours (`scene::fill_light_tables`).
    pub lamp_table_c: [Dword; 256],
    /// +0x1400 (0x636518): 14 bytes between the lamp tables and `view_mode`; nothing reads or writes them (a gap, probably padding).
    pub unknown_400: Bytes<0xE>,
    /// +0x140e (0x636526): `word_636526`: the view window's mode (`scene::choose_view_window`, `scene::full_view_window`).
    pub view_mode: Word,
    /// +0x1410 (0x636528): 24 dwords after `view_mode`; nothing in the Rust port reads or writes them (a gap).
    pub unknown_410: [Dword; 24],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl LightTables {
    /// The piece with the image's first values (the owner `scene::owner::Scene` holds it).
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x635118, the first byte of the piece.
pub const START: usize = 0x635118;
/// IDA 0x636588, the end of the piece.
pub const END: usize = 0x636588;

const _: () = assert!(END - START == core::mem::size_of::<LightTables>());

/// Where IDA 0x635118 is now.
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(
        &crate::scene::owner::SCENE.light_tables,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::scene::owner::SCENE.light_tables,
            START,
            END,
        );
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(LightTables, lamp_table_a), 0x635518 - START);
        assert_eq!(offset_of!(LightTables, car_light_table), 0x635D18 - START);
        assert_eq!(offset_of!(LightTables, lamp_table_c), 0x636118 - START);
        assert_eq!(offset_of!(LightTables, view_mode), 0x636526 - START);
        assert_eq!(offset_of!(LightTables, unknown_410), 0x636528 - START);
    }
}
