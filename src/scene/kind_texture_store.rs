//! The texture kind tables (IDA 0x5D0AE0 .. 0x5D2AE8): the two running totals of memory (video, system) asked for while
//! allocating, then g_kindFirstTextures and g_kindFirstSlots, 0x400 dwords each, indexed by kind slot
//! (single entries such as 0x5D0DEC, 0x5D0E10, 0x5D1E10 are indexes into them). Names are readings
//! unless IDA's.

use crate::cells::ZeroCells;
use crate::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct KindTextureTables {
    /// (0x5D0AE0): `dword_5D0AE0`: bytes of video memory asked for so far by the texture allocation, a running sum of each needed texture's level size; only logged ("VMEMREQ") and left in the game's data (`scene::try_allocate_textures`).
    pub video_memory_asked: PlainDword,
    /// (0x5D0AE4): `dword_5D0AE4`: bytes of system memory asked for so far, the same sum over the textures kept in system memory; only logged ("SMEMREQ") (`scene::try_allocate_textures`).
    pub system_memory_asked: PlainDword,
    /// (0x5D0AE8): `g_kindFirstTextures`: each kind slot's first texture; reached in place and by index (`scene::textures`, `scene::texture_levels`, `race::mud`, `race::frame_views`, `effects::`). Reset to 0xFF by `scene::reset_texture_tables`.
    pub kind_first_textures: [PlainDword; 1024],
    /// (0x5D1AE8): `g_kindFirstSlots`: each kind slot's first texture slot (`scene::look_up_texture`, `scene::draw_polygon`, `hud::set_batch_texture`, `race::frame_flush`). Reset to 0xFF by `scene::reset_texture_tables`.
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
