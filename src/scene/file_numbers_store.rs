//! The object and picture loaders' file numbers (IDA 0x60AD20 .. 0x60AF0C; phase 7, typed): the
//! .ro3 / .lwo name numbers and paths, the picture number and texture. The path strings between
//! them stay unknown_* bytes, reached by address. `#[repr(C)]` with the image's layout over the
//! race block's bytes.
//! Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
pub struct ObjectFileNumbers {
    /// +0x0 (0x60AD20): not named.
    pub unknown_0: Dword,
    /// +0x4 (0x60AD24): `dword_60AD24`: .ro3 name number (`scene::set_object_file_names`).
    pub object_ro3_number: Dword,
    /// +0x8 (0x60AD28): `dword_60AD28`: .ro3 path (`scene::set_object_file_names`).
    pub object_ro3_path: Dword,
    /// +0xc (0x60AD2C): `dword_60AD2C`: .lwo path (`scene::set_object_file_names`).
    pub object_lwo_path: Dword,
    /// +0x10 (0x60AD30): `dword_60AD30`: .lwo name number (`scene::set_object_file_names`).
    pub object_lwo_number: Dword,
    /// +0x14 (0x60AD34): not named.
    pub unknown_14: [Dword; 116],
    /// +0x1e4 (0x60AF04): `dword_60AF04`: the picture number (`scene::load_texture_picture`).
    pub picture_number: Dword,
    /// +0x1e8 (0x60AF08): `dword_60AF08`: the texture (`scene::load_texture_picture`).
    pub picture_texture: Dword,
}

/// The piece: the race block's bytes at IDA 0x60AD20 seen as `ObjectFileNumbers`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = ObjectFileNumbers;
    fn deref(&self) -> &ObjectFileNumbers {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(piece_addr() as *const ObjectFileNumbers) }
    }
}

pub static FILE_NUMBERS: InRaceBlock = InRaceBlock;

/// IDA 0x60AD20, the first byte of the piece.
pub const START: usize = 0x60AD20;
/// IDA 0x60AF0C, the end of the piece.
pub const END: usize = 0x60AF0C;

const _: () = assert!(END - START == core::mem::size_of::<ObjectFileNumbers>());

/// Where IDA 0x60AD20 is now (in the race block).
pub fn piece_addr() -> usize {
    crate::race::store::home(START)
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
            offset_of!(ObjectFileNumbers, object_ro3_number),
            0x60AD24 - START
        );
    }
}
