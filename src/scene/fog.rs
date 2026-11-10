//! A view's fog (frame plan F4): what `race::set_up_fog` works out for the view and its
//! drawing reads, one in each view's slot (`race::view_scratch`) and one for the front end.
//!
//! The original kept it in the light state for every view at once: `g_fogTable` (0x63661C,
//! 0x1000 bytes), its length `g_fogTableLength` (0x63761C), the next step `dword_636610` and
//! the far value `dword_6365F4` (the glare's alpha). Each view set it up before reading it,
//! and the overlays drawn after a view read that view's own. A length of 0 (no fog set up
//! since the start) made `scene::fog_polygon` read the byte before the table, which nothing
//! in the game writes: 0 here. The near value (`dword_6365F0`) and the noise step
//! (`dword_62DAF0`) were stored but never read: not kept.

use crate::session::{Bytes, Dword};

/// The table's room in bytes.
pub const TABLE_LEN: usize = 0x1000;

pub struct Fog {
    /// The fog's far value (`dword_6365F4`).
    pub far: Dword,
    /// The fog's next step (`dword_636610`), what every point gets with the view kind 4.
    pub next: Dword,
    /// The table's length in bytes (`g_fogTableLength`).
    pub length: Dword,
    /// The fog levels by depth (`g_fogTable`).
    table: Bytes<TABLE_LEN>,
    /// The fog's colour (`DRAW.fog_colour`, `g_fogColour` 0x624000), each view's own.
    pub colour: Dword,
}

impl Fog {
    pub const fn new() -> Self {
        Self {
            far: Dword::new(0),
            next: Dword::new(0),
            length: Dword::new(0),
            table: Bytes::new([0; TABLE_LEN]),
            colour: Dword::new(0),
        }
    }

    /// The table's first bytes from `levels` (the rest kept, as the original's copy).
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

    /// `scene::clear_fog` (0x44FC19): no fog: the next step and the whole table 0 (the
    /// length kept).
    pub fn clear(&self) {
        self.next.set(0);
        for n in 0..TABLE_LEN {
            self.table.set(n, 0);
        }
    }
}
