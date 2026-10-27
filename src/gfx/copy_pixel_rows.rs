//! `gfx::copy_pixel_rows` (0x407540): the picture converter of the pixel formats that need
//! no conversion (numbers 5 and 6 in the converter table): copies rows of dwords.
//!
//! In:  esi = from, edi = to, ecx = dwords in a row, edx = rows, ebp = what is taken off
//!      edi after each row (so the rows on the destination side lie `4 * ecx - ebp` apart)
//! Out: with esi equal to edi nothing happens and nothing changes. Otherwise ebx = ecx as
//!      it came, edx = 0, esi and edi moved on past the rows; eax, ecx, ebp preserved. The
//!      flags are not reproduced.
//! Reached only through the converter table `dword_4F5288`: `gfx::convert_picture` calls
//! it with esi = edi (the picture is already what the surface wants: on the owner's 32-bit
//! screen that is every picture), `sub_41A828` with a locked texture as the destination.
//! The name is a reading.

use crate::framework::rva;

pub const COPY_PIXEL_ROWS: usize = rva(0x407540);

// ---- Rust version ----

// None (phase 12, 2026-10-27): its one call left, `gfx::convert_picture`'s, has esi = edi,
// where the original does nothing; the picture buffer is Rust's and not reached by address.
