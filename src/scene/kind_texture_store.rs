//! The texture kind tables (IDA 0x5D0AE0 .. 0x5D2AE8): the two running totals of textures asked for while
//! allocating, then g_kindFirstTextures and g_kindFirstSlots, 0x400 dwords each, indexed by kind slot
//! (single entries such as 0x5D0DEC, 0x5D0E10, 0x5D1E10 are indexes into them). `#[repr(C)]` with the image's
//! layout, over the race block's bytes. Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
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

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static KIND_TEXTURES: KindTextureTables =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x5D0AE0, the first byte of the piece.
pub const START: usize = 0x5D0AE0;
/// IDA 0x5D2AE8, the end of the piece.
pub const END: usize = 0x5D2AE8;

const _: () = assert!(END - START == core::mem::size_of::<KindTextureTables>());

/// Where IDA 0x5D0AE0 is now (the piece's static).
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(std::ptr::from_ref(&KIND_TEXTURES))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&KIND_TEXTURES, START, END);
    }

    #[test]
    fn piece_has_its_home() {
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
