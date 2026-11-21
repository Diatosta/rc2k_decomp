//! The lighting tables: the colours the lights pick from (the light table, two lamp tables, the car
//! light table) (`scene::owner::Scene::light_tables`). A plain record (`session::DrawPlain`),
//! written on the game's thread between frames and read there and, as a copy made when a drawing
//! scope opens, by the drawing parts through their `scene::workers::DrawReads`.

use crate::memory::session::{DrawPlain, PlainRead};

/// The four tables of 256 colours.
#[derive(Clone)]
pub struct LightTables {
    /// 256 colours (`scene::vertex_light`, `physics::car_lighting`, `race::course_faces`).
    pub light_table: [u32; 256],
    /// Lamp A's 256 colours.
    pub lamp_table_a: [u32; 256],
    /// Lamp B's 256 colours.
    pub lamp_table_b: [u32; 256],
    /// 256 colours (`scene::add_car_light`, `race::frame_draw`).
    pub car_light_table: [u32; 256],
}

impl LightTables {
    /// The store with its first values (every colour 0).
    pub const fn new() -> Self {
        Self {
            light_table: [0; 256],
            lamp_table_a: [0; 256],
            lamp_table_b: [0; 256],
            car_light_table: [0; 256],
        }
    }
}

/// The tables as one reader finds them: the game's thread's own (read through the store when used),
/// or a drawing part's copy.
#[derive(Clone, Copy)]
pub enum LightTablesRef<'a> {
    Store(&'a DrawPlain<LightTables>),
    Copy(&'a LightTables),
}

impl LightTablesRef<'_> {
    /// Entry `index` of the light table.
    pub fn light(self, index: usize) -> u32 {
        match self {
            Self::Store(store) => store.borrow().light_table[index],
            Self::Copy(tables) => tables.light_table[index],
        }
    }

    /// Entry `index` of lamp A's table.
    pub fn lamp_a(self, index: usize) -> u32 {
        match self {
            Self::Store(store) => store.borrow().lamp_table_a[index],
            Self::Copy(tables) => tables.lamp_table_a[index],
        }
    }

    /// Entry `index` of lamp B's table.
    pub fn lamp_b(self, index: usize) -> u32 {
        match self {
            Self::Store(store) => store.borrow().lamp_table_b[index],
            Self::Copy(tables) => tables.lamp_table_b[index],
        }
    }

    /// Entry `index` of the car light table.
    pub fn car_light(self, index: usize) -> u32 {
        match self {
            Self::Store(store) => store.borrow().car_light_table[index],
            Self::Copy(tables) => tables.car_light_table[index],
        }
    }
}

impl<'a> From<&'a PlainRead<LightTables>> for LightTablesRef<'a> {
    fn from(copy: &'a PlainRead<LightTables>) -> Self {
        Self::Copy(copy)
    }
}
