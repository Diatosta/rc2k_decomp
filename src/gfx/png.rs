/// The buffer the info block is in. The shared scratch (`files::scratch::SCRATCH`) holds first
/// the file's first 0x80 bytes, then what the decoder says about the picture, laid out as the `INFO_` offsets below.
pub fn info_buffer(system: &crate::system::owner::System) -> &crate::session::PlainBytes<65536> {
    &system.work().files.buffer
}

/// The info's dword at `offset` (`INFO_WIDTH` or `INFO_HEIGHT`).
pub fn info_dword(offset: usize, system: &crate::system::owner::System) -> u32 {
    system.work().files.buffer.get_u32(offset)
}

/// The info's byte at `offset`.
pub fn info_byte(offset: usize, system: &crate::system::owner::System) -> u8 {
    system.work().files.buffer.get(offset)
}

/// What `decode_png` leaves behind in the info buffer: a byte that is 1 if the file has a
/// palette (a PLTE chunk), else 0 (`INFO_HAS_PALETTE`); the picture's width and height, dwords
/// (`INFO_WIDTH`, `INFO_HEIGHT`); and a palette of 0x300 bytes, three an entry (`INFO_PALETTE`).
/// Only the first byte is written, and the
/// palette only when the file has one: the rest is whatever was there (in the original, the
/// work area of zlib, which it lays over this block).
pub const INFO_HAS_PALETTE: usize = 0;
pub const INFO_WIDTH: usize = 4;
pub const INFO_HEIGHT: usize = 8;
pub const INFO_PALETTE: usize = 12;
