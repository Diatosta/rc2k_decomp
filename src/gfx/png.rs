//! What the PNG reader's parts share: the block the decoder describes a picture in, and PNG
//! files for their self-tests.

use crate::framework::rva;

/// `buf` (0x93E8B0): first the file's first 0x80 bytes, then what the decoder says about
/// the picture, a `PngInfo`.
pub const INFO_RVA: usize = rva(0x93E8B0);

/// What `decode_png` leaves behind. Only the first byte of `has_palette` is written, and the
/// palette only when the file has one: the rest is whatever was there (in the original, the
/// work area of zlib, which it lays over this block).
#[repr(C)]
pub struct PngInfo {
    /// 1 if the file has a palette (a PLTE chunk), else 0.
    pub has_palette: u8,
    pub unset: [u8; 3],
    pub width: u32,
    pub height: u32,
    /// Three bytes an entry.
    pub palette: [u8; 0x300],
}

// ---- for the self-tests ----
