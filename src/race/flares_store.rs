//! The flares' tables (IDA 0x63F58C .. 0x64098C; phase 7, typed): five of the six flare
//! tables (the first, the places' x, is `LEVEL_GRID.flare_place_x`, 0x63F18C): the places'
//! y and z, the other light, the size and the object number, each 0x400 bytes. The tables
//! hold 0x100 flares but `race::frame_draw::add_flare` runs on to 0x400 (a slip of the
//! original's: it writes into the tables that follow, so its code reaches them by address).
//! `#[repr(C)]` with the image's layout, over the race block's bytes (`FLARE_TABLES`).
//! Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
pub struct FlareTables {
    /// +0x0 (0x63F58C): `dword_63F58C`: the flares' y (`race::frame_draw::add_flare`, `draw_flare`).
    pub place_y: [Dword; 256],
    /// +0x400 (0x63F98C): `dword_63F98C`: the flares' z.
    pub place_z: [Dword; 256],
    /// +0x800 (0x63FD8C): `dword_63FD8C`: the flares' colour (`g_otherLight`).
    pub other_light: [Dword; 256],
    /// +0xc00 (0x64018C): `dword_64018C`: the flares' size.
    pub size: [Dword; 256],
    /// +0x1000 (0x64058C): `dword_64058C`: the flares' object number.
    pub object_number: [Dword; 256],
}

/// The piece: the race block's bytes at IDA 0x63F58C seen as `FlareTables`.
pub struct InBlock;

impl core::ops::Deref for InBlock {
    type Target = FlareTables;
    fn deref(&self) -> &FlareTables {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(piece_addr() as *const FlareTables) }
    }
}

pub static FLARE_TABLES: InBlock = InBlock;

/// IDA 0x63F58C, the first byte of the piece.
pub const START: usize = 0x63F58C;
/// IDA 0x64098C, the end of the piece.
pub const END: usize = 0x64098C;

const _: () = assert!(END - START == core::mem::size_of::<FlareTables>());

/// Where IDA 0x63F58C is now.
pub fn piece_addr() -> usize {
    crate::moved::place(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            piece_addr() + (END - START) - 1
        );
        assert_eq!(offset_of!(FlareTables, place_z), 0x63F98C - START);
        assert_eq!(offset_of!(FlareTables, other_light), 0x63FD8C - START);
        assert_eq!(offset_of!(FlareTables, size), 0x64018C - START);
        assert_eq!(offset_of!(FlareTables, object_number), 0x64058C - START);
    }
}
