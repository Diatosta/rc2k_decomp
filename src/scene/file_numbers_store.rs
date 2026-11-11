//! The object and picture loaders' file numbers (IDA 0x60AD20 .. 0x60AF0C): the
//! .ro3 / .lwo name numbers and paths, the picture number and texture. The path strings between
//! them stay unknown_* bytes, reached by address. `#[repr(C)]` with the image's layout over the
//! race block's bytes.
//! Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ObjectFileNumbers {
    /// +0x0 (0x60AD20): the dword before the file numbers; nothing reads or writes it (a gap).
    pub unknown_0: Dword,
    /// +0x4 (0x60AD24): `dword_60AD24`: .ro3 name number, kept as the text's IDA address within `path_texts` (the original kept its address; `scene::set_object_file_names`).
    pub object_ro3_number: Dword,
    /// +0x8 (0x60AD28): `dword_60AD28`: .ro3 path, kept as the text's IDA address within `path_texts` (the original kept its address; `scene::set_object_file_names`).
    pub object_ro3_path: Dword,
    /// +0xc (0x60AD2C): `dword_60AD2C`: .lwo path, kept as the text's IDA address within `path_texts` (the original kept its address; `scene::set_object_file_names`).
    pub object_lwo_path: Dword,
    /// +0x10 (0x60AD30): `dword_60AD30`: .lwo name number, kept as the text's IDA address within `path_texts` (the original kept its address; `scene::set_object_file_names`).
    pub object_lwo_number: Dword,
    /// +0x14 (0x60AD34): the path texts the stored text addresses above point into: the folder, map, info and car-icon file names (`texts`, reached by IDA address through `text_at`; the dwords are only the storage, 464 bytes of NUL-ended strings).
    pub path_texts: [Dword; 116],
    /// +0x1e4 (0x60AF04): `dword_60AF04`: the picture number (`scene::load_texture_picture`).
    pub picture_number: Dword,
    /// +0x1e8 (0x60AF08): `dword_60AF08`: the texture (`scene::load_texture_picture`).
    pub picture_texture: Dword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl ObjectFileNumbers {
    /// The piece with the image's first values (the owner `scene::owner::Scene` holds it).
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// The path texts in `path_texts`, by IDA's address (for `ObjectFileNumbers::text_at`).
pub mod texts {
    pub const FOLDER_0_RO3: usize = 0x60AD4E;
    pub const FOLDER_0_LWO: usize = 0x60AD62;
    pub const FOLDER_1_RO3: usize = 0x60AD76;
    pub const FOLDER_1_LWO: usize = 0x60AD8A;
    pub const FOLDER_2_RO3: usize = 0x60AD9E;
    pub const FOLDER_2_LWO: usize = 0x60ADB7;
    pub const FOLDER_3_RO3: usize = 0x60ADD0;
    pub const FOLDER_3_LWO: usize = 0x60ADE9;
    pub const ERROR_LIST_PATH: usize = 0x60AE02;
    pub const MAP_PATH: usize = 0x60AE12;
    pub const NUMBER_PLACE_A: usize = 0x60AE0F;
    pub const NUMBER_PLACE_B: usize = 0x60AE1D;
    pub const NUMBER_PLACE_C: usize = 0x60AE2F;
    pub const MAP_KEY: usize = 0x60AE24;
    pub const TEXTURE_INFO_A: usize = 0x60AE36;
    pub const MARK_INFO: usize = 0x60AE47;
    pub const TEXTURE_INFO_B: usize = 0x60AE58;
    pub const OBJECTS_INFO: usize = 0x60AE69;
    pub const CARS_INFO: usize = 0x60AE7A;
    pub const EXTRAS_INFO: usize = 0x60AE8B;
    pub const FRONT_INFO: usize = 0x60AEA1;
    pub const CAR_ICON_0: usize = 0x60AEB7;
    pub const CAR_ICON_1: usize = 0x60AECA;
    pub const CAR_ICON_2: usize = 0x60AEDD;
    pub const CAR_ICON_3: usize = 0x60AEF0;
}

/// A path text in `path_texts`: read up to its NUL and written to inside the 464 bytes the
/// texts occupy, its bounds.
pub struct PathText<'a> {
    numbers: &'a ObjectFileNumbers,
    /// The text's first byte, within `path_texts`.
    from: usize,
}

impl ObjectFileNumbers {
    /// The text at IDA address `ida`.
    pub fn text_at(&self, ida: usize) -> PathText<'_> {
        assert!((0x60AD34..0x60AF04).contains(&ida));
        PathText {
            numbers: self,
            from: ida - 0x60AD34,
        }
    }
}

impl crate::session::TextPlace for PathText<'_> {
    fn text(&self, _what: &str) -> Result<Vec<u8>, crate::errors::BadInput> {
        let mut rest = vec![0; TEXTS_LEN - self.from];
        crate::pointers::load_cells(self.numbers, TEXTS_AT + self.from, &mut rest)?;
        match rest.iter().position(|&c| c == 0) {
            Some(n) => Ok(rest[..n].to_vec()),
            None => Err(crate::errors::BadInput("a text with no end in its buffer")),
        }
    }

    fn write_text_at(
        &self,
        at: usize,
        bytes: &[u8],
        _what: &str,
    ) -> Result<(), crate::errors::BadInput> {
        let from = self.from.saturating_add(at);
        if from.saturating_add(bytes.len()) > TEXTS_LEN {
            return Err(crate::errors::BadInput("a write past the path texts"));
        }
        crate::pointers::store_cells(self.numbers, TEXTS_AT + from, bytes)
    }
}

/// `path_texts`' place in the piece and its length in bytes.
const TEXTS_AT: usize = core::mem::offset_of!(ObjectFileNumbers, path_texts);
const TEXTS_LEN: usize = 4 * 116;

/// IDA 0x60AD20, the first byte of the piece.
pub const START: usize = 0x60AD20;
/// IDA 0x60AF0C, the end of the piece.
pub const END: usize = 0x60AF0C;

const _: () = assert!(END - START == core::mem::size_of::<ObjectFileNumbers>());

/// Where IDA 0x60AD20 is now (its own static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(
        &crate::scene::owner::SCENE.file_numbers,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::scene::owner::SCENE.file_numbers,
            START,
            END,
        );
    }

    #[test]
    fn piece_has_its_home() {
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
