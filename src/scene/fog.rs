//! A view's fog: what `race::set_up_fog` works out for the view and its drawing reads, one in
//! each view's slot (`race::view_scratch`) and one for the front end.
//!
//! It holds the fog table (0x1000 bytes), its length, the next step and the far value (the
//! glare's alpha). Each view sets it up before reading it, and the overlays drawn after a view
//! read that view's own. A length of 0 (no fog set up since the start) makes
//! `scene::fog_polygon` read the byte before the table, which nothing in the game writes: 0.

use crate::race::plain_slot::{Merge, PlainSlot, merge_run, merge_value};

/// The table's room in bytes.
pub const TABLE_LEN: usize = 0x1000;

/// A view's fog values, plain.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FogValues {
    /// The fog's far value (the glare's alpha).
    pub far: u32,
    /// The fog's next step, what every point gets with the view kind 4.
    pub next: u32,
    /// The table's length in bytes.
    pub length: u32,
    /// The fog levels by depth.
    table: [u8; TABLE_LEN],
    /// The fog's colour (`DRAW.fog_colour`), each view's own.
    pub colour: u32,
}

/// A view's fog.
pub type Fog = PlainSlot<FogValues>;

impl FogValues {
    pub const fn new() -> Self {
        Self {
            far: 0,
            next: 0,
            length: 0,
            table: [0; TABLE_LEN],
            colour: 0,
        }
    }

    /// The table's first bytes from `levels` (the rest kept).
    pub fn set_levels(&mut self, levels: &[u8]) {
        for (n, &b) in levels.iter().take(TABLE_LEN).enumerate() {
            self.table[n] = b;
        }
    }

    /// The level at `place`; the place before the table (`u32::MAX`, a length of 0) is 0.
    pub fn level(&self, place: u32) -> u8 {
        match place as usize {
            n if n < TABLE_LEN => self.table[n],
            _ => 0,
        }
    }

    /// `scene::clear_fog`: no fog: the next step and the whole table 0 (the
    /// length kept).
    pub fn clear(&mut self) {
        self.next = 0;
        self.table = [0; TABLE_LEN];
    }
}

impl Merge for FogValues {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_value(&mut self.far, &now.far, &start.far);
        merge_value(&mut self.next, &now.next, &start.next);
        merge_value(&mut self.length, &now.length, &start.length);
        merge_run(&mut self.table, &now.table, &start.table);
        merge_value(&mut self.colour, &now.colour, &start.colour);
    }
}
