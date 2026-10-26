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
//! ported). So on a machine of today it is never called, and its self-test is all it has.
//! Manual patch with a private copy (0x33 bytes, nothing to relocate): the loop comes back
//! to the third byte, inside what the patch overwrites.
//! Refuses a row or a row count of 0 (the original counts through all 32 bits). The name
//! is a reading.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, EDI, EDX, ESI, Regs, rva};
use crate::gfx::harness::pixels_entry;
use crate::screens::harness::Left;

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
                let components = (from as *const [u8; 4]).read_unaligned();
                let [low, high] = rc2k_core::gfx::pixels::to_4444(components);
                (to as *mut u8).write(low);
                (to.wrapping_add(1) as *mut u8).write(high);
                ax = u16::from_le_bytes([high, components[3] & 0xF0]);
            }
            from = from.wrapping_add(4);
            to = to.wrapping_add(2);
        }
        to = to.wrapping_sub(back);
    }
    Ok((from, to, ax))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (from, to, ax) = unsafe { components_to_4444_rows(r.esi, r.edi, r.ecx, r.edx, r.ebp) }?;
    Ok(Left::nothing()
        .with(EAX, r.eax & 0xFFFF_0000 | ax as u32)
        .with(EBX, r.ecx)
        .with(EDX, 0)
        .with(ESI, from)
        .with(EDI, to))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    pixels_entry(r, &rust)
}
