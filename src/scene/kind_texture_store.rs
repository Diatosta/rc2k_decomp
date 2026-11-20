//! The texture kind tables, indexed by kind slot: the kinds' first texture slots (`KindSlots`, what
//! the drawing parts read, a lent store) and their first textures (`KindTextures`, plain fields
//! only the game's thread reaches, `scene::owner::Textures::first_textures`).

use crate::memory::cells::ZeroCells;
use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct KindSlots {
    /// Each kind slot's first texture slot (`scene::look_up_texture`, `scene::draw_polygon`,
    /// `hud::set_batch_texture`, `race::frame_flush`). Reset to 0xFF by
    /// `scene::reset_texture_tables`.
    pub kind_first_slots: [PlainDword; 1024],
}

impl KindSlots {
    /// The store with its first values (everything starts at zero).
    pub const fn new() -> Self {
        <Self as ZeroCells>::ZERO
    }
}

/// Each kind slot's first texture; reached in place and by index (`scene::textures`,
/// `scene::texture_levels`, `race::mud`, `race::frame_views`, `effects::`). Reset to 0xFF by
/// `scene::reset_texture_tables`.
pub type KindTextures = [u32; 1024];
