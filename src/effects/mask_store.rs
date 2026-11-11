//! The marks' mask pictures' variables, IDA
//! 0x934CC8 .. 0x934F30: which ground picture (`load_mask_pictures`), the lists the mask blocks
//! are taken on, the pointers to the blocks, the paths of the mask pictures, and the number of
//! the picture of each kind. Four statics, because the image keeps its cells on three different
//! alignments (`MaskBlocks` on 2 more than a multiple of 4, `PictureCopy` on 1 more): each
//! struct starts at its first cell, so that offsets inside stay as in the image.

use crate::session::{Bytes, Dword};

/// The ground picture's name (IDA 0x934CC8 .. 0x934D6E). The offsets are from its start.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct GroundNames {
    /// +0x0: `g_marksGround`: which of the six names (0 .. 5)
    pub ground: Dword,
    /// +0x4: `g_marksGroundPath`: the offset of the chosen name in `names` (the original kept its address; set by `load_mask_pictures`)
    pub ground_path: Dword,
    /// +0x8: `aFilesMSnow1Pcx` ..: `files\m\snow1.pcx`, `mud1`, `dust1`, `mud1`, `mud1`, `gumbo0`, each NUL-ended, then zeros
    pub names: Bytes<160>,
}

/// The ground picture's names; the first values are the image's.
impl GroundNames {
    pub const fn new() -> Self {
        GroundNames {
            ground: Dword::new(0x0),
            ground_path: Dword::new(0x0),
            names: Bytes::new([
                0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x73, 0x6E, 0x6F, 0x77, 0x31, 0x2E,
                0x70, 0x63, 0x78, 0x00, 0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x6D, 0x75,
                0x64, 0x31, 0x2E, 0x70, 0x63, 0x78, 0x00, 0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D,
                0x5C, 0x64, 0x75, 0x73, 0x74, 0x31, 0x2E, 0x70, 0x63, 0x78, 0x00, 0x66, 0x69, 0x6C,
                0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x6D, 0x75, 0x64, 0x31, 0x2E, 0x70, 0x63, 0x78, 0x00,
                0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x6D, 0x75, 0x64, 0x31, 0x2E, 0x70,
                0x63, 0x78, 0x00, 0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x67, 0x75, 0x6D,
                0x62, 0x6F, 0x30, 0x2E, 0x70, 0x63, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]),
        }
    }
}

impl GroundNames {
    pub const START: usize = 0x934CC8;
    pub const LEN: usize = 0xA6;
}

/// The mask blocks' lists and pointers (IDA 0x934D6E .. 0x934E29, on an address 2 more than a multiple of 4 in the image). The offsets are from its start.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MaskBlocks {
    /// +0x0: `dword_934D6E`: the last mask switch seen by `swap_mask` (-1 once the masks are loaded)
    pub seen: Dword,
    /// +0x4: `dword_934D72`: the block `swap_mask` last drew from (the game's address;
    /// since phase 12 `Mask::Plain` 1 or `Mask::Bordered` 2, 0 none yet)
    pub chosen: Dword,
    /// +0x8: `stru_934D76`: the list the first mask block is taken on
    pub list_a: [Dword; 4],
    /// +0x18: Not named: 48 bytes (a gap, or unreferenced data).
    pub _gap_18: Bytes<48>,
    /// +0x48: `g_marksMaskBlock`: the first mask block (0x20000 bytes; the game's
    /// address, unused since the masks are Rust's, `MASK`)
    pub mask: Dword,
    /// +0x4C: `g_marksBMaskList`: the list the second mask block is taken on
    pub list_b: [Dword; 4],
    /// +0x5C: Not named: 48 bytes (a gap, or unreferenced data).
    pub _gap_5c: Bytes<48>,
    /// +0x8C: `g_marksBMaskBlock`: the second mask block (0x20000 bytes; the game's
    /// address, unused since the masks are Rust's, `BMASK`)
    pub bmask: Dword,
    /// +0x90: `aRseRseRseRseRs`: "rse " repeated, then a NUL (unreferenced; the next list's head starts inside it)
    pub junk: Bytes<44>,
}

/// The mask blocks' lists; the first values are the image's.
impl MaskBlocks {
    pub const fn new() -> Self {
        MaskBlocks {
            seen: Dword::new(0xFFFFFFFF),
            chosen: Dword::new(0x0),
            list_a: [
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
            ],
            _gap_18: Bytes::new([0; 48]),
            mask: Dword::new(0x0),
            list_b: [
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
            ],
            _gap_5c: Bytes::new([0; 48]),
            bmask: Dword::new(0x0),
            junk: Bytes::new([
                0x72, 0x73, 0x65, 0x20, 0x72, 0x73, 0x65, 0x20, 0x72, 0x73, 0x65, 0x20, 0x72, 0x73,
                0x65, 0x20, 0x72, 0x73, 0x65, 0x20, 0x72, 0x73, 0x65, 0x20, 0x72, 0x73, 0x65, 0x20,
                0x72, 0x73, 0x65, 0x20, 0x72, 0x73, 0x65, 0x20, 0x72, 0x73, 0x65, 0x20, 0x72, 0x73,
                0x65, 0x00,
            ]),
        }
    }
}

/// The mask and surface stores (step 11, owned state), handed by reference to the code that
/// reads them; `effects::owner::Effects::masks` holds the one instance.
pub struct Masks {
    pub ground_names: GroundNames,
    pub blocks: MaskBlocks,
    /// `g_marksMaskBlock`'s pixels, on `blocks.list_a`.
    pub mask: crate::memory::owned::Owned<MaskPixels>,
    /// `g_marksBMaskBlock`'s pixels, on `blocks.list_b`.
    pub bmask: crate::memory::owned::Owned<MaskPixels>,
    pub picture_copy: PictureCopy,
    /// The masks' work block (`masks::MASK_WORK` before).
    pub work: crate::memory::owned::Owned<Vec<u32>>,
    pub list: crate::effects::set_up_surface::MaskList,
    pub surface: crate::effects::surface_store::SurfaceTables,
    pub edge_settings: crate::effects::edges::EdgeSettings,
    pub edge_mask: crate::effects::edges::EdgeMask,
    pub kind_boxes: crate::effects::edges::KindBoxes,
    pub vars: crate::effects::mask_vars::MaskVars,
}

impl Masks {
    pub const fn new() -> Self {
        Masks {
            ground_names: GroundNames::new(),
            blocks: MaskBlocks::new(),
            mask: crate::memory::owned::Owned::new("no marks mask"),
            bmask: crate::memory::owned::Owned::new("no marks bordered mask"),
            picture_copy: PictureCopy::new(),
            work: crate::memory::owned::Owned::new("no mask work"),
            list: crate::effects::set_up_surface::MaskList::new(),
            surface: crate::effects::surface_store::SurfaceTables::new(),
            edge_settings: crate::effects::edges::EdgeSettings::new(),
            edge_mask: crate::effects::edges::EdgeMask::new(),
            kind_boxes: crate::effects::edges::KindBoxes::new(),
            vars: crate::effects::mask_vars::MaskVars::new(),
        }
    }
}

/// A mask's pixels (the game's 0x20000-byte block: 256 x 128 dwords), Rust's since phase
/// 12; taken counted on its list (`load_mask_pictures`), given back by `tidy_marks`.
pub type MaskPixels = Vec<u32>;
/// The dwords of a mask block.
pub const MASK_PIXELS: usize = 0x8000;

/// Which mask `swap_mask` draws from (`MASK_BLOCKS.chosen`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mask {
    Plain = 1,
    Bordered = 2,
}

impl Mask {
    pub fn pixels(self, masks: &Masks) -> &crate::memory::owned::Owned<MaskPixels> {
        match self {
            Mask::Plain => &masks.mask,
            Mask::Bordered => &masks.bmask,
        }
    }
}

impl MaskBlocks {
    pub const START: usize = 0x934D6E;
    pub const LEN: usize = 0xBB;
}

/// The picture copy's list and pointer, and the mask paths (IDA 0x934E29 .. 0x934EA8, on an address 1 more than a multiple of 4 in the image). The offsets are from its start.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct PictureCopy {
    /// +0x0: the list the 0x80000-byte picture copy is taken on (its head starts inside `aRseRseRseRseRs`: the image's first values are "se" and zeros)
    pub list: [Dword; 4],
    /// +0x10: Not named: 48 bytes (a gap, or unreferenced data).
    pub _gap_10: Bytes<48>,
    /// +0x40: `g_marksPictureCopy`: the picture copy block (0x80000 bytes; the game's
    /// address, unused since the copy is Rust's, phase 12)
    pub copy: Dword,
    /// +0x44: `files\m\mask00.pcx` at +0, `files\m\bmask00.pcx` at +0x13, `files\m\bdata00.pcx` at +0x25 (`byte_934E79` .. the digits: +0xC, +0x20, +0x34)
    pub paths: Bytes<60>,
}

/// The picture copy's list; the first values are the image's.
impl PictureCopy {
    pub const fn new() -> Self {
        PictureCopy {
            list: [
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
            ],
            _gap_10: Bytes::new([0; 48]),
            copy: Dword::new(0x0),
            paths: Bytes::new([
                0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x6D, 0x61, 0x73, 0x6B, 0x30, 0x30,
                0x2E, 0x70, 0x63, 0x78, 0x00, 0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x62,
                0x6D, 0x61, 0x73, 0x6B, 0x30, 0x30, 0x2E, 0x70, 0x63, 0x78, 0x00, 0x66, 0x69, 0x6C,
                0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x62, 0x64, 0x61, 0x74, 0x61, 0x30, 0x30, 0x2E, 0x70,
                0x63, 0x78, 0x00, 0x00,
            ]),
        }
    }
}

impl PictureCopy {
    pub const START: usize = 0x934E29;
    pub const LEN: usize = 0x7F;
}

/// The picture number of each kind (IDA 0x934EA8 .. 0x934F30). The offsets are from its start.
#[repr(C)]
pub struct KindPictures {
    /// +0x0: `g_marksKindPictureNumbers`: per kind, the number in the mask and picture paths' two digits
    pub numbers: [u32; 34],
}

/// The pictures' numbers; the first values are the image's.
pub static KIND_PICTURES: KindPictures = KindPictures {
    numbers: [
        0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11,
        0x12, 0x13, 0x14, 0x15, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16,
        0x0,
    ],
};

#[cfg(test)]
impl KindPictures {
    pub const START: usize = 0x934EA8;
    pub const LEN: usize = 0x88;
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    /// The first `len` bytes of `v` (its fields' bytes: the padding past them is never read,
    /// it holds no value).
    fn bytes<T>(v: &T, len: usize) -> Vec<u8> {
        assert!(len <= size_of::<T>());
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..len).map(|i| unsafe { p.add(i).read() }).collect()
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
            bytes(
                &crate::effects::owner::EFFECTS.masks.ground_names,
                GroundNames::LEN
            ) == image(GroundNames::START, GroundNames::LEN)
        );
        assert!(
            bytes(
                &crate::effects::owner::EFFECTS.masks.blocks,
                MaskBlocks::LEN
            ) == image(MaskBlocks::START, MaskBlocks::LEN)
        );
        assert!(
            bytes(
                &crate::effects::owner::EFFECTS.masks.picture_copy,
                PictureCopy::LEN
            ) == image(PictureCopy::START, PictureCopy::LEN)
        );
        assert!(
            bytes(&KIND_PICTURES, KindPictures::LEN)
                == image(KindPictures::START, KindPictures::LEN)
        );
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
