//! `gfx::components_to_4444_rows` (0x40750D): the plain picture converter for pixel format
//! number 4, two bytes a pixel with four bits for each component.
//!
//! In:  esi = from (four bytes a pixel), edi = to, ecx = pixels in a row, edx = rows, ebp =
//!      what is taken off edi after each row
//! Out: ebx = ecx as it came, edx = 0, esi and edi moved on past the rows, ax = what the
//!      last pixel left there (al its second byte, ah the top four bits of its fourth
//!      component); the rest of eax, ecx and ebp preserved. The flags are not reproduced.
//! A pixel's first byte is the top four bits of its first component in the low half and
//! of its second in the high half; its second byte the same of the third and fourth.
//! Reached only through the converter table `dword_4F5288`, where `gfx::install_converters`
//! puts it when the processor has no MMX; with MMX `sub_407276` stands there instead (not
//! ported). So on a machine of today it is never called.
//! Refuses a row or a row count of 0 (the original counts through all 32 bits). The name
//! is a reading.

use crate::errors::BadInput;
use crate::framework::rva;

pub const COMPONENTS_TO_4444_ROWS: usize = rva(0x40750D);

// ---- Rust version ----

/// Where the two sides end up and what the last pixel leaves in ax: (esi, edi, ax).
pub unsafe fn components_to_4444_rows(
    from: u32,
    to: u32,
    row_pixels: u32,
    rows: u32,
    back: u32,
) -> Result<(u32, u32, u16), BadInput> {
    if row_pixels == 0 || rows == 0 {
        return Err(BadInput("no pixels to convert"));
    }
    let (mut from, mut to, mut ax) = (from, to, 0u16);
    for _ in 0..rows {
        for _ in 0..row_pixels {
            unsafe {
                // Converted: `rc2k_core::gfx::pixels::to_4444`.
                let components = (crate::pointers::host(from) as *const [u8; 4]).read_unaligned();
                let [low, high] = rc2k_core::gfx::pixels::to_4444(components);
                crate::pointers::host(to).write(low);
                crate::pointers::host(to.wrapping_add(1)).write(high);
                ax = u16::from_le_bytes([high, components[3] & 0xF0]);
            }
            from = from.wrapping_add(4);
            to = to.wrapping_add(2);
        }
        to = to.wrapping_sub(back);
    }
    Ok((from, to, ax))
}
