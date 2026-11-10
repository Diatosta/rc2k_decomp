//! `gfx::copy_pixel_rows` (0x407540): the picture converter of the pixel formats that need
//! no conversion (numbers 5 and 6 in the converter table): copies rows of dwords.
//!
//! Reached only through the converter table `dword_4F5288`: `gfx::convert_picture` calls
//! it with esi = edi (the picture is already what the surface wants: on the owner's 32-bit
//! screen that is every picture), 0x41A828 with a locked texture as the destination.
//! The name is a reading.

pub const COPY_PIXEL_ROWS: u32 = 0x407540;

// None: its one call left, `gfx::convert_picture`'s, has esi = edi,
// where the original does nothing; the picture buffer is Rust's and not reached by address.
