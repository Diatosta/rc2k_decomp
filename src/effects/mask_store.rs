//! The marks' mask pictures' variables: which ground picture (`load_mask_pictures`), the
//! lists the mask blocks are taken on, the blocks, the paths of the mask pictures, and the
//! number of the picture of each kind. Four statics, so that each struct starts at its first
//! cell and its offsets stay as listed.

use crate::memory::session::{PlainBytes, PlainDword};

/// The ground picture's name. The offsets are from its start.
#[derive(rc2k_derive::Cells)]
pub struct GroundNames {
    /// +0x0: which of the six names (0 .. 5)
    pub ground: PlainDword,
    /// +0x4: the offset of the chosen name in `names` (set by `load_mask_pictures`)
    pub ground_path: PlainDword,
    /// +0x8: `files\m\snow1.pcx`, `mud1`, `dust1`, `mud1`, `mud1`, `gumbo0`, each NUL-ended, then
    /// zeros
    pub names: PlainBytes<160>,
}

/// The ground picture's names.
impl GroundNames {
    pub const fn new() -> Self {
        GroundNames {
            ground: PlainDword::new(0x0),
            ground_path: PlainDword::new(0x0),
            names: PlainBytes::new([
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

/// The mask blocks' lists and pointers. The offsets are from its start.
#[derive(rc2k_derive::Cells)]
pub struct MaskBlocks {
    /// +0x0: the last mask switch seen by `swap_mask` (-1 once the masks are loaded)
    pub seen: PlainDword,
    /// +0x4: the block `swap_mask` last drew from (`Mask::Plain` 1 or `Mask::Bordered` 2, 0
    /// none yet)
    pub chosen: PlainDword,
    /// +0x8: the list the first mask block is taken on
    pub list_a: crate::memory::ListCells,
    /// +0x48: the first mask block (0x20000 bytes; unused: the pixels are `Masks::mask`)
    pub mask: PlainDword,
    /// +0x4C: the list the second mask block is taken on
    pub list_b: crate::memory::ListCells,
    /// +0x8C: the second mask block (0x20000 bytes; unused: the pixels are `Masks::bmask`)
    pub bmask: PlainDword,
    /// +0x90: "rse " repeated, then a NUL (unreferenced)
    pub junk: PlainBytes<44>,
}

/// The mask blocks' lists.
impl MaskBlocks {
    pub const fn new() -> Self {
        MaskBlocks {
            seen: PlainDword::new(0xFFFFFFFF),
            chosen: PlainDword::new(0x0),
            list_a: crate::memory::ListCells::empty(),
            mask: PlainDword::new(0x0),
            list_b: crate::memory::ListCells::empty(),
            bmask: PlainDword::new(0x0),
            junk: PlainBytes::new([
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
    /// The first mask's pixels, on `blocks.list_a`.
    pub mask: crate::memory::owned::Owned<MaskPixels>,
    /// The second mask's pixels, on `blocks.list_b`.
    pub bmask: crate::memory::owned::Owned<MaskPixels>,
    pub picture_copy: PictureCopy,
    /// The masks' work block.
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

/// A mask's pixels (256 x 128 dwords); taken counted on its list (`load_mask_pictures`),
/// given back by `tidy_marks`.
type MaskPixels = Vec<u32>;
/// The dwords of a mask block.
pub(super) const MASK_PIXELS: usize = 0x8000;

/// Which mask `swap_mask` draws from (`MaskBlocks.chosen`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mask {
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

/// The picture copy's list and pointer, and the mask paths. The offsets are from its start.
#[derive(rc2k_derive::Cells)]
pub struct PictureCopy {
    /// +0x0: the list the 0x80000-byte picture copy is taken on
    pub list: crate::memory::ListCells,
    /// +0x40: the picture copy block (0x80000 bytes; unused, the copy is Rust's)
    pub copy: PlainDword,
    /// +0x44: `files\m\mask00.pcx` at +0, `files\m\bmask00.pcx` at +0x13, `files\m\bdata00.pcx` at
    /// +0x25 (the digits: +0xC, +0x20, +0x34)
    pub paths: PlainBytes<60>,
}

/// The picture copy's list.
impl PictureCopy {
    pub const fn new() -> Self {
        PictureCopy {
            list: crate::memory::ListCells::empty(),
            copy: PlainDword::new(0x0),
            paths: PlainBytes::new([
                0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x6D, 0x61, 0x73, 0x6B, 0x30, 0x30,
                0x2E, 0x70, 0x63, 0x78, 0x00, 0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x62,
                0x6D, 0x61, 0x73, 0x6B, 0x30, 0x30, 0x2E, 0x70, 0x63, 0x78, 0x00, 0x66, 0x69, 0x6C,
                0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x62, 0x64, 0x61, 0x74, 0x61, 0x30, 0x30, 0x2E, 0x70,
                0x63, 0x78, 0x00, 0x00,
            ]),
        }
    }
}

/// The picture number of each kind. The offsets are from its start.
pub struct KindPictures {
    /// +0x0: per kind, the number in the mask and picture paths' two digits
    pub numbers: [u32; 34],
}

/// The pictures' numbers.
pub(super) static KIND_PICTURES: KindPictures = KindPictures {
    numbers: [
        0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11,
        0x12, 0x13, 0x14, 0x15, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16,
        0x0,
    ],
};
