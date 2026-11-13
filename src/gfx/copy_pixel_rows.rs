//! The picture converter of the pixel formats that need no conversion (numbers 5 and 6 in the
//! converter table): copies rows of dwords.

pub const COPY_PIXEL_ROWS: u32 = 0x407540;

// None: its one call left, `gfx::convert_picture`'s, has source = destination, where it does
// nothing; the picture buffer is Rust's and not reached by address.
