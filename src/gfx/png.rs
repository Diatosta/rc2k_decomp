/// The shared scratch (`files::scratch::SCRATCH`, 0x93E8B0) holds first the file's first
/// 0x80 bytes, then what the decoder says about the picture, a `PngInfo`.
pub fn info() -> *mut PngInfo {
    crate::files::scratch::SCRATCH.buffer.as_ptr().cast()
}

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
