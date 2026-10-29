//! The texture kind tables (IDA 0x5D0AE0 .. 0x5D2AE8): the two running totals of textures asked for while
//! allocating, then g_kindFirstTextures and g_kindFirstSlots, 0x400 dwords each, indexed by kind slot
//! (single entries such as 0x5D0DEC, 0x5D0E10, 0x5D1E10 are indexes into them). `#[repr(C)]` with the image's
//! layout, over the race block's bytes. Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
pub struct KindTextureTables {
    /// +0x0 (0x5D0AE0): `dword_5D0AE0`: a running total of the textures asked for (`scene::try_allocate_textures`).
    pub textures_asked: Dword,
    /// +0x4 (0x5D0AE4): `dword_5D0AE4`: the second running total kept beside it (`scene::try_allocate_textures`; a reading).
    pub textures_asked_other: Dword,
    /// +0x8 (0x5D0AE8): `g_kindFirstTextures`: each kind slot's first texture; reached in place and by index (`scene::textures`, `scene::texture_levels`, `race::mud`, `race::frame_views`, `effects::`). Reset to 0xFF by `scene::reset_texture_tables`.
    pub kind_first_textures: [Dword; 1024],
    /// +0x1008 (0x5D1AE8): `g_kindFirstSlots`: each kind slot's first texture slot (`scene::look_up_texture`, `scene::draw_polygon`, `hud::set_batch_texture`, `race::frame_flush`). Reset to 0xFF by `scene::reset_texture_tables`.
    pub kind_first_slots: [Dword; 1024],
}

/// The piece: the race block's bytes at IDA 0x5D0AE0 seen as `KindTextureTables`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = KindTextureTables;
    fn deref(&self) -> &KindTextureTables {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host(piece_addr()) as *const KindTextureTables) }
    }
}

pub static KIND_TEXTURES: InRaceBlock = InRaceBlock;

/// IDA 0x5D0AE0, the first byte of the piece.
pub const START: usize = 0x5D0AE0;
/// IDA 0x5D2AE8, the end of the piece.
pub const END: usize = 0x5D2AE8;

const _: () = assert!(END - START == core::mem::size_of::<KindTextureTables>());

/// Where IDA 0x5D0AE0 is now (in the race block).
pub fn piece_addr() -> u32 {
    crate::race::store::home(START) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(
            offset_of!(KindTextureTables, textures_asked),
            0x5D0AE0 - START
        );
    }
}
