//! The lighting tables: the colours the lights pick from (the light table, two lamp tables, the car
//! light table, a third lamp table), the view window's mode and the words around it
//! (`scene::owner::Scene::light_tables`). Plain cells: a drawing store (`session::DrawShared`),
//! written on the game's thread between frames and read by the drawing parts through their
//! `scene::workers::DrawReads`.

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainDword, PlainWord};

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
    /// The view window's mode (`scene::choose_view_window`, `scene::full_view_window`).
    pub view_mode: PlainWord,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl LightTables {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self { ..ZeroCells::ZERO }
    }
}
