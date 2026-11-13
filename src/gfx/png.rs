//! What the PNG decoder says about a picture.

use crate::errors::BadInput;

/// What `decode_png` says about the picture it decoded: whether the file has a palette (a
/// PLTE chunk), the width and the height, and the palette's 0x300 bytes, three an entry.
/// `decode_png` sets the fields it learns from the file; the others stay as the shared file
/// scratch had them (see `PngInfo::leftover`).
pub struct PngInfo {
    pub has_palette: bool,
    pub width: u32,
    pub height: u32,
    pub palette: [u8; PALETTE_LEN],
}

pub const PALETTE_LEN: usize = 0x300;

/// Where the original laid the info block in the shared file scratch.
const SCRATCH_PALETTE_AT: usize = 12;

impl PngInfo {
    /// What the decoder leaves unwritten: the first bytes of the shared file scratch, which
    /// hold the file's head (`read_png` has just read it there) or an earlier user's bytes:
    /// the first byte tells a palette if it is 1, then the width, the height and the palette's
    /// 0x300 bytes. Read once, before the decoder runs; nothing writes the scratch between.
    pub fn leftover(scratch: &crate::session::PlainBytes<65536>) -> Result<PngInfo, BadInput> {
        let what = "the PNG info's leftover";
        let mut palette = [0u8; PALETTE_LEN];
        palette.copy_from_slice(&scratch.read_at(SCRATCH_PALETTE_AT, PALETTE_LEN, what)?);
        Ok(PngInfo {
            has_palette: scratch.get(0) == 1,
            width: scratch.get_u32(4),
            height: scratch.get_u32(8),
            palette,
        })
    }
}
