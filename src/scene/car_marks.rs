//! A car's marks (frame plan F4): the textures of its number, digits, plates and decals
//! that its faces' kinds pick (`scene::list_object_face`), set for each car as it is drawn
//! (`race::car_number_textures`) and carried in the drawing's context.
//!
//! The original kept them in `g_carMarkTextures` (0x5CD688, 0x100 dwords) for every drawing
//! at once: each car set them, drew, and cleared the first 0x40 after (`race::clear_car_marks`,
//! 0x46202A), and outside a race `scene::draw_shown_object` set all 0x100 to the menus' kind
//! bytes before every object. Entries 0x40 .. 0xFF were never anything but 0 (the image's
//! and the menus' value). So: a car's own marks while it is drawn, none between cars, the
//! menus' outside a race. One case differs: in a race, what is drawn before its first car's
//! clear read the menus' 999 in entries 6, 7 and 9 (none here); a face's byte cannot be 999,
//! so such a face of those kinds was dropped, kept here if its byte is 0.

use crate::session::Dword;

/// The marks a car's faces can pick.
pub const MARKS: usize = 0x40;

pub struct CarMarks([Dword; MARKS]);

impl CarMarks {
    pub const fn new() -> Self {
        Self([const { Dword::new(0) }; MARKS])
    }

    /// The menus' (`rc2k_core::scene::shown::kind_bytes`, `scene::clear_kind_bytes`): 0 but
    /// the seventh, eighth and tenth, 999.
    const fn menus() -> Self {
        let mut cells = [const { Dword::new(0) }; MARKS];
        cells[6] = Dword::new(999);
        cells[7] = Dword::new(999);
        cells[9] = Dword::new(999);
        Self(cells)
    }

    /// Mark `n` (below `MARKS`).
    pub fn mark(&self, n: usize) -> &Dword {
        &self.0[n]
    }

    /// What a face of kind `kind` compares its byte with: its mark, 0 past them.
    pub fn kind_byte(&self, kind: u32) -> u32 {
        self.0.get(kind as usize).map_or(0, Dword::get)
    }
}

/// Between cars in a race.
pub static NONE: CarMarks = CarMarks::new();

/// Outside a race.
pub static MENUS: CarMarks = CarMarks::menus();
