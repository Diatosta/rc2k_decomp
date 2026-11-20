//! The texture kind tables: the kinds' first textures and first slots, 0x400 dwords each, indexed by kind
//! slot.

use crate::memory::cells::ZeroCells;
use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct KindTextureTables {
    /// Each kind slot's first texture; reached in place and by index (`scene::textures`,
    /// `scene::texture_levels`, `race::mud`, `race::frame_views`, `effects::`). Reset to 0xFF
    /// by `scene::reset_texture_tables`.
    pub kind_first_textures: [PlainDword; 1024],
    /// Each kind slot's first texture slot (`scene::look_up_texture`, `scene::draw_polygon`,
    /// `hud::set_batch_texture`, `race::frame_flush`). Reset to 0xFF by
    /// `scene::reset_texture_tables`.
    pub kind_first_slots: [PlainDword; 1024],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl KindTextureTables {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        <Self as ZeroCells>::ZERO
    }
}
