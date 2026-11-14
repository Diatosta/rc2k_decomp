//! The lighting tables: the colours the lights pick from (the light table, two lamp tables, the car
//! light table, a third lamp table), the view window's mode and the words around it
//! (`scene::owner::Scene::light_tables`). Plain cells: a drawing store (`session::DrawShared`),
//! written on the game's thread between frames and read by the drawing parts through their
//! `workers::DrawReads`.

use crate::cells::ZeroCells;
use crate::session::{PlainBytes, PlainDword, PlainWord};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct LightTables {
    /// 256 colours (`scene::vertex_light`, `physics::car_lighting`, `race::course_faces`).
    pub light_table: [PlainDword; 256],
    /// Lamp A's 256 colours.
    pub lamp_table_a: [PlainDword; 256],
    /// Lamp B's 256 colours.
    pub lamp_table_b: [PlainDword; 256],
    /// 256 colours (`scene::add_car_light`, `race::frame_draw`).
    pub car_light_table: [PlainDword; 256],
    /// A third lamp table of 256 colours (`scene::fill_light_tables`).
    pub lamp_table_c: [PlainDword; 256],
    /// 14 bytes between the lamp tables and `view_mode`; nothing reads or writes them (a gap,
    /// probably padding).
    pub unknown_400: PlainBytes<0xE>,
    /// The view window's mode (`scene::choose_view_window`, `scene::full_view_window`).
    pub view_mode: PlainWord,
    /// 24 dwords after `view_mode`; nothing reads or writes them (a gap).
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
