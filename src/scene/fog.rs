//! A view's fog: what `race::set_up_fog` works out for the view and its drawing reads, one in
//! each view's slot (`race::view_scratch`) and one for the front end.
//!
//! It holds the fog table (0x1000 bytes), its length, the next step and the far value (the
//! glare's alpha). Each view sets it up before reading it, and the overlays drawn after a view
//! read that view's own. A length of 0 (no fog set up since the start) makes
//! `scene::fog_polygon` read the byte before the table, which nothing in the game writes: 0.

use crate::memory::session::{PlainBytes, PlainDword};

/// The table's room in bytes.
pub const TABLE_LEN: usize = 0x1000;

#[derive(rc2k_derive::Cells)]
pub struct Fog {
    /// The fog's far value (the glare's alpha).
    pub far: PlainDword,
    /// The fog's next step, what every point gets with the view kind 4.
    pub next: PlainDword,
    /// The table's length in bytes.
    pub length: PlainDword,
    /// The fog levels by depth.
    table: PlainBytes<TABLE_LEN>,
    /// The fog's colour (`DRAW.fog_colour`), each view's own.
    pub colour: PlainDword,
}

impl Fog {
    pub const fn new() -> Self {
        Self {
            far: PlainDword::new(0),
            next: PlainDword::new(0),
            length: PlainDword::new(0),
            table: PlainBytes::new([0; TABLE_LEN]),
            colour: PlainDword::new(0),
        }
    }

    /// The table's first bytes from `levels` (the rest kept).
    pub fn set_levels(&self, levels: &[u8]) {
        for (n, &b) in levels.iter().take(TABLE_LEN).enumerate() {
            self.table.set(n, b);
        }
    }

    /// The level at `place`; the place before the table (`u32::MAX`, a length of 0) is 0.
    pub fn level(&self, place: u32) -> u8 {
        match place as usize {
            n if n < TABLE_LEN => self.table.get(n),
            _ => 0,
        }
    }

    /// `scene::clear_fog`: no fog: the next step and the whole table 0 (the
    /// length kept).
    pub fn clear(&self) {
        self.next.set(0);
        for n in 0..TABLE_LEN {
            self.table.set(n, 0);
        }
    }
}
