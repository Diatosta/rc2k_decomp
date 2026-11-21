//! The texture kind tables, indexed by kind slot: the kinds' first texture slots (`KindSlots`, a plain
//! record, `session::DrawPlain`: written on the game's thread between frames, read there and, as a copy
//! made when a drawing scope opens, by the drawing parts) and their first textures (`KindTextures`,
//! plain fields only the game's thread reaches, `scene::owner::Textures::first_textures`).

use crate::memory::session::{DrawPlain, PlainRead};

/// Each kind slot's first texture slot (`scene::look_up_texture`, `scene::draw_polygon`,
/// `hud::set_batch_texture`, `race::frame_flush`). Reset to 0xFF by `scene::reset_texture_tables`.
#[derive(Clone)]
pub struct KindSlots {
    pub kind_first_slots: [u32; 1024],
}

impl KindSlots {
    /// The store with its first values (everything starts at zero).
    pub const fn new() -> Self {
        Self {
            kind_first_slots: [0; 1024],
        }
    }
}

/// The slots as one reader finds them: the game's thread's own (read through the store when used), or
/// a drawing part's copy.
#[derive(Clone, Copy)]
pub enum KindSlotsRef<'a> {
    Store(&'a DrawPlain<KindSlots>),
    Copy(&'a KindSlots),
}

impl KindSlotsRef<'_> {
    /// Kind slot `kind`'s first texture slot.
    pub fn first_slot(self, kind: usize) -> u32 {
        match self {
            Self::Store(store) => store.borrow().kind_first_slots[kind],
            Self::Copy(slots) => slots.kind_first_slots[kind],
        }
    }
}

impl<'a> From<&'a PlainRead<KindSlots>> for KindSlotsRef<'a> {
    fn from(copy: &'a PlainRead<KindSlots>) -> Self {
        Self::Copy(copy)
    }
}

/// Each kind slot's first texture; reached in place and by index (`scene::textures`,
/// `scene::texture_levels`, `race::mud`, `race::frame_views`, `effects::`). Reset to 0xFF by
/// `scene::reset_texture_tables`.
pub type KindTextures = [u32; 1024];
