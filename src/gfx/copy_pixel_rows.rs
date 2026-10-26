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
//! Refuses a row count of 0 when something would be copied (the original counts through all 32
//! bits). The name is a reading.

use crate::errors::BadInput;
use crate::framework::rva;

pub const COPY_PIXEL_ROWS: usize = rva(0x407540);

// ---- Rust version ----

/// Where the two sides end up: (esi, edi). None if nothing was to be done.
pub unsafe fn copy_pixel_rows(
    from: u32,
    to: u32,
    row_dwords: u32,
    rows: u32,
    back: u32,
) -> Result<Option<(u32, u32)>, BadInput> {
    if from == to {
        return Ok(None);
    }
    if rows == 0 {
        return Err(BadInput("no rows to copy"));
    }
    let (mut from, mut to) = (from, to);
    for _ in 0..rows {
        // Dword by dword and forwards, as `rep movsd` goes, whatever overlaps.
        for _ in 0..row_dwords {
            unsafe {
                (crate::pointers::host(to) as *mut u32)
                    .write_unaligned((crate::pointers::host(from) as *const u32).read_unaligned());
            }
            from = from.wrapping_add(4);
            to = to.wrapping_add(4);
        }
        to = to.wrapping_sub(back);
    }
    Ok(Some((from, to)))
}
