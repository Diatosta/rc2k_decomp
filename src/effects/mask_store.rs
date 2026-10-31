//! The marks' mask pictures' variables, out of the marks' block (phase 7, typed state), IDA
//! 0x934CC8 .. 0x934F30: which ground picture (`load_mask_pictures`), the lists the mask blocks
//! are taken on, the pointers to the blocks, the paths of the mask pictures, and the number of
//! the picture of each kind. Four statics, because the image keeps its cells on three different
//! alignments (`MaskBlocks` on 2 more than a multiple of 4, `PictureCopy` on 1 more): each
//! struct starts at its first cell, so that offsets inside stay as in the image.

use crate::memory::MemoryList;
use crate::session::{Bytes, Dword, Shared};

/// The ground picture's name (IDA 0x934CC8 .. 0x934D6E; phase 7, typed state). The offsets are from its start.
#[repr(C)]
pub struct GroundNames {
    /// +0x0: `g_marksGround`: which of the six names (0 .. 5)
    pub ground: Dword,
    /// +0x4: `g_marksGroundPath`: the address of the chosen name (set by `load_mask_pictures`)
    pub ground_path: Dword,
    /// +0x8: `aFilesMSnow1Pcx` ..: `files\m\snow1.pcx`, `mud1`, `dust1`, `mud1`, `mud1`, `gumbo0`, each NUL-ended, then zeros
    pub names: Bytes<158>,
}

/// The ground picture's names; the first values are the image's.
pub static GROUND_NAMES: GroundNames = GroundNames {
    ground: Dword::new(0x0),
    ground_path: Dword::new(0x0),
    names: Bytes::new([
        0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x73, 0x6E, 0x6F, 0x77, 0x31, 0x2E, 0x70,
        0x63, 0x78, 0x00, 0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x6D, 0x75, 0x64, 0x31,
        0x2E, 0x70, 0x63, 0x78, 0x00, 0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x64, 0x75,
        0x73, 0x74, 0x31, 0x2E, 0x70, 0x63, 0x78, 0x00, 0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D,
        0x5C, 0x6D, 0x75, 0x64, 0x31, 0x2E, 0x70, 0x63, 0x78, 0x00, 0x66, 0x69, 0x6C, 0x65, 0x73,
        0x5C, 0x6D, 0x5C, 0x6D, 0x75, 0x64, 0x31, 0x2E, 0x70, 0x63, 0x78, 0x00, 0x66, 0x69, 0x6C,
        0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x67, 0x75, 0x6D, 0x62, 0x6F, 0x30, 0x2E, 0x70, 0x63, 0x78,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]),
};

impl GroundNames {
    pub const START: usize = 0x934CC8;
    pub const LEN: usize = 0xA6;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const GROUND_NAMES) as usize
    }
}

/// The mask blocks' lists and pointers (IDA 0x934D6E .. 0x934E29, on an address 2 more than a multiple of 4 in the image; phase 7, typed state). The offsets are from its start.
#[repr(C)]
pub struct MaskBlocks {
    /// +0x0: `dword_934D6E`: the last mask switch seen by `swap_mask` (-1 once the masks are loaded)
    pub seen: Dword,
    /// +0x4: `dword_934D72`: the block `swap_mask` last drew from (the game's address;
    /// since phase 12 `Mask::Plain` 1 or `Mask::Bordered` 2, 0 none yet)
    pub chosen: Dword,
    /// +0x8: `stru_934D76`: the list the first mask block is taken on
    pub list_a: Shared<MemoryList>,
    /// +0x18: Not named: 48 bytes (a gap, or unreferenced data).
    pub _gap_18: Bytes<48>,
    /// +0x48: `g_marksMaskBlock`: the first mask block (0x20000 bytes; the game's
    /// address, unused since the masks are Rust's, `MASK`)
    pub mask: Dword,
    /// +0x4C: `g_marksBMaskList`: the list the second mask block is taken on
    pub list_b: Shared<MemoryList>,
    /// +0x5C: Not named: 48 bytes (a gap, or unreferenced data).
    pub _gap_5c: Bytes<48>,
    /// +0x8C: `g_marksBMaskBlock`: the second mask block (0x20000 bytes; the game's
    /// address, unused since the masks are Rust's, `BMASK`)
    pub bmask: Dword,
    /// +0x90: `aRseRseRseRseRs`: "rse " repeated, then a NUL (unreferenced; the next list's head starts inside it)
    pub junk: Bytes<43>,
}

/// The mask blocks' lists; the first values are the image's.
pub static MASK_BLOCKS: MaskBlocks = MaskBlocks {
    seen: Dword::new(0xFFFFFFFF),
    chosen: Dword::new(0x0),
    list_a: Shared::new(MemoryList {
        head: 0x0,
        total: 0x0,
        cursor: 0x0,
        remaining: 0x0,
    }),
    _gap_18: Bytes::new([0; 48]),
    mask: Dword::new(0x0),
    list_b: Shared::new(MemoryList {
        head: 0x0,
        total: 0x0,
        cursor: 0x0,
        remaining: 0x0,
    }),
    _gap_5c: Bytes::new([0; 48]),
    bmask: Dword::new(0x0),
    junk: Bytes::new([
        0x72, 0x73, 0x65, 0x20, 0x72, 0x73, 0x65, 0x20, 0x72, 0x73, 0x65, 0x20, 0x72, 0x73, 0x65,
        0x20, 0x72, 0x73, 0x65, 0x20, 0x72, 0x73, 0x65, 0x20, 0x72, 0x73, 0x65, 0x20, 0x72, 0x73,
        0x65, 0x20, 0x72, 0x73, 0x65, 0x20, 0x72, 0x73, 0x65, 0x20, 0x72, 0x73, 0x65,
    ]),
};

/// A mask's pixels (the game's 0x20000-byte block: 256 x 128 dwords), Rust's since phase
/// 12; taken counted on its list (`load_mask_pictures`), given back by `tidy_marks`.
pub type MaskPixels = Vec<u32>;
/// The dwords of a mask block.
pub const MASK_PIXELS: usize = 0x8000;
/// `g_marksMaskBlock`'s pixels, on `MASK_BLOCKS.list_a`.
pub static MASK: crate::memory::owned::Owned<MaskPixels> =
    crate::memory::owned::Owned::new("no marks mask");
/// `g_marksBMaskBlock`'s pixels, on `MASK_BLOCKS.list_b`.
pub static BMASK: crate::memory::owned::Owned<MaskPixels> =
    crate::memory::owned::Owned::new("no marks bordered mask");

/// Which mask `swap_mask` draws from (`MASK_BLOCKS.chosen`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mask {
    Plain = 1,
    Bordered = 2,
}

impl Mask {
    pub fn pixels(self) -> &'static crate::memory::owned::Owned<MaskPixels> {
        match self {
            Mask::Plain => &MASK,
            Mask::Bordered => &BMASK,
        }
    }
}

impl MaskBlocks {
    pub const START: usize = 0x934D6E;
    pub const LEN: usize = 0xBB;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const MASK_BLOCKS) as usize
    }
}

/// The picture copy's list and pointer, and the mask paths (IDA 0x934E29 .. 0x934EA8, on an address 1 more than a multiple of 4 in the image; phase 7, typed state). The offsets are from its start.
#[repr(C)]
pub struct PictureCopy {
    /// +0x0: the list the 0x80000-byte picture copy is taken on (its head starts inside `aRseRseRseRseRs`: the image's first values are "se" and zeros)
    pub list: Shared<MemoryList>,
    /// +0x10: Not named: 48 bytes (a gap, or unreferenced data).
    pub _gap_10: Bytes<48>,
    /// +0x40: `g_marksPictureCopy`: the picture copy block (0x80000 bytes; the game's
    /// address, unused since the copy is Rust's, phase 12)
    pub copy: Dword,
    /// +0x44: `files\m\mask00.pcx` at +0, `files\m\bmask00.pcx` at +0x13, `files\m\bdata00.pcx` at +0x25 (`byte_934E79` .. the digits: +0xC, +0x20, +0x34)
    pub paths: Bytes<59>,
}

/// The picture copy's list; the first values are the image's.
pub static PICTURE_COPY: PictureCopy = PictureCopy {
    list: Shared::new(MemoryList {
        head: 0x0,
        total: 0x0,
        cursor: 0x0,
        remaining: 0x0,
    }),
    _gap_10: Bytes::new([0; 48]),
    copy: Dword::new(0x0),
    paths: Bytes::new([
        0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x6D, 0x61, 0x73, 0x6B, 0x30, 0x30, 0x2E,
        0x70, 0x63, 0x78, 0x00, 0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x62, 0x6D, 0x61,
        0x73, 0x6B, 0x30, 0x30, 0x2E, 0x70, 0x63, 0x78, 0x00, 0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C,
        0x6D, 0x5C, 0x62, 0x64, 0x61, 0x74, 0x61, 0x30, 0x30, 0x2E, 0x70, 0x63, 0x78, 0x00,
    ]),
};

impl PictureCopy {
    pub const START: usize = 0x934E29;
    pub const LEN: usize = 0x7F;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const PICTURE_COPY) as usize
    }
}

/// The picture number of each kind (IDA 0x934EA8 .. 0x934F30; phase 7, typed state). The offsets are from its start.
#[repr(C)]
pub struct KindPictures {
    /// +0x0: `g_marksKindPictureNumbers`: per kind, the number in the mask and picture paths' two digits
    pub numbers: [Dword; 34],
}

/// The pictures' numbers; the first values are the image's.
pub static KIND_PICTURES: KindPictures = KindPictures {
    numbers: [
        Dword::new(0x0),
        Dword::new(0x1),
        Dword::new(0x2),
        Dword::new(0x3),
        Dword::new(0x4),
        Dword::new(0x5),
        Dword::new(0x6),
        Dword::new(0x7),
        Dword::new(0x8),
        Dword::new(0x9),
        Dword::new(0xA),
        Dword::new(0xB),
        Dword::new(0xC),
        Dword::new(0xD),
        Dword::new(0xE),
        Dword::new(0xF),
        Dword::new(0x10),
        Dword::new(0x11),
        Dword::new(0x12),
        Dword::new(0x13),
        Dword::new(0x14),
        Dword::new(0x15),
        Dword::new(0x16),
        Dword::new(0x16),
        Dword::new(0x16),
        Dword::new(0x16),
        Dword::new(0x16),
        Dword::new(0x16),
        Dword::new(0x16),
        Dword::new(0x16),
        Dword::new(0x16),
        Dword::new(0x16),
        Dword::new(0x16),
        Dword::new(0x0),
    ],
};

impl KindPictures {
    pub const START: usize = 0x934EA8;
    pub const LEN: usize = 0x88;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const KIND_PICTURES) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn the_statics_start_as_the_games() {
        let image = crate::moved::image_bytes;
        // The structs end on an odd byte in the image; their sizes round up to 4 (the padding
        // is past the range and nothing reaches it).
        assert_eq!(
            size_of::<GroundNames>(),
            GroundNames::LEN.next_multiple_of(4)
        );
        assert_eq!(size_of::<MaskBlocks>(), MaskBlocks::LEN.next_multiple_of(4));
        assert_eq!(
            size_of::<PictureCopy>(),
            PictureCopy::LEN.next_multiple_of(4)
        );
        assert_eq!(size_of::<KindPictures>(), KindPictures::LEN);
        assert!(
            bytes(&GROUND_NAMES)[..GroundNames::LEN] == image(GroundNames::START, GroundNames::LEN)
        );
        assert!(
            bytes(&MASK_BLOCKS)[..MaskBlocks::LEN] == image(MaskBlocks::START, MaskBlocks::LEN)
        );
        assert!(
            bytes(&PICTURE_COPY)[..PictureCopy::LEN] == image(PictureCopy::START, PictureCopy::LEN)
        );
        assert!(bytes(&KIND_PICTURES) == image(KindPictures::START, KindPictures::LEN));
    }

    #[test]
    fn the_fields_are_where_the_game_has_them() {
        assert_eq!(
            GroundNames::START + offset_of!(GroundNames, names),
            0x934CD0
        );
        assert_eq!(MaskBlocks::START + offset_of!(MaskBlocks, list_a), 0x934D76);
        assert_eq!(MaskBlocks::START + offset_of!(MaskBlocks, mask), 0x934DB6);
        assert_eq!(MaskBlocks::START + offset_of!(MaskBlocks, list_b), 0x934DBA);
        assert_eq!(MaskBlocks::START + offset_of!(MaskBlocks, bmask), 0x934DFA);
        assert_eq!(PictureCopy::START + offset_of!(PictureCopy, copy), 0x934E69);
        assert_eq!(
            PictureCopy::START + offset_of!(PictureCopy, paths),
            0x934E6D
        );
        assert_eq!(MaskBlocks::START + MaskBlocks::LEN, PictureCopy::START);
        assert_eq!(PictureCopy::START + PictureCopy::LEN, KindPictures::START);
    }
}
