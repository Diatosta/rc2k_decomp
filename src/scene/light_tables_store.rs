//! The lighting tables (IDA 0x635118 .. 0x636588): the colours the lights
//! pick from (the light table, two lamp tables, the car light table, a third lamp table),
//! the view window's mode and the words around it. `#[repr(C)]` with the image's layout,
//! over the race block's bytes (`scene::owner::Scene::light_tables`). Plain cells: a drawing
//! store (`session::DrawShared`), written on the game's thread between frames and read by
//! the drawing parts through their `workers::DrawReads`. Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::{PlainBytes, PlainDword, PlainWord};

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct LightTables {
    /// +0x0 (0x635118): `g_lightTable`: 256 colours (`scene::vertex_light`, `physics::car_lighting`, `race::course_faces`).
    pub light_table: [PlainDword; 256],
    /// +0x400 (0x635518): `dword_635518`: lamp A's 256 colours.
    pub lamp_table_a: [PlainDword; 256],
    /// +0x800 (0x635918): `dword_635918`: lamp B's 256 colours.
    pub lamp_table_b: [PlainDword; 256],
    /// +0xc00 (0x635D18): `g_carLightTable`: 256 colours (`scene::add_car_light`, `race::frame_draw`).
    pub car_light_table: [PlainDword; 256],
    /// +0x1000 (0x636118): `unk_636118`: a third lamp table of 256 colours (`scene::fill_light_tables`).
    pub lamp_table_c: [PlainDword; 256],
    /// +0x1400 (0x636518): 14 bytes between the lamp tables and `view_mode`; nothing reads or writes them (a gap, probably padding).
    pub unknown_400: PlainBytes<0xE>,
    /// +0x140e (0x636526): `word_636526`: the view window's mode (`scene::choose_view_window`, `scene::full_view_window`).
    pub view_mode: PlainWord,
    /// +0x1410 (0x636528): 24 dwords after `view_mode`; nothing in the Rust port reads or writes them (a gap).
    pub unknown_410: [PlainDword; 24],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl LightTables {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self {
            unknown_410: {
                let mut a: [PlainDword; 24] = ZeroCells::ZERO;
                a[0] = PlainDword::new(0xC0000);
                a
            },
            ..ZeroCells::ZERO
        }
    }
}
