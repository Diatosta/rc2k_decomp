//! `gfx::blend_palette` (0x405F0A): makes a palette that lies between two others.
//!
//! In:  esi = first palette, edx = second palette, edi = where the result goes,
//!      ecx = number of entries, eax = how far towards the second (0 .. 256),
//!      ebx = pixel format
//! Out: every register preserved
//! For each entry both colours are taken apart with `pixel_to_components`; each of the three
//! low components becomes first + ((second - first) * eax >> 8), as a byte; and the result
//! goes back through `components_to_pixel`. The fourth component is not computed: it is
//! whatever the top byte of the result's entry held before, which only matters to a format
//! with a fourth mask.
//! One caller, `text::blend_font`.
//! Refuses a count of 0 (the original goes through all 32 bits) and null pointers.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::gfx::block::game_write;
use crate::gfx::pixel_format::{PixelFormat, SCRATCH_RVA};

// ---- Rust version ----

pub unsafe fn blend_palette(
    first: *const u32,
    second: *const u32,
    out: *mut u32,
    count: u32,
    towards_second: u32,
    format: &PixelFormat,
) -> Result<(), BadInput> {
    if count == 0 {
        return Err(BadInput(
            "no entries: the original goes through all 32 bits",
        ));
    }
    if first.is_null() || second.is_null() || out.is_null() {
        return Err(BadInput("a null palette"));
    }
    for n in 0..count as usize {
        unsafe {
            // Converted: `rc2k_core::gfx::pixels::blend_entry`; the scratch bytes as
            // `components_to_pixel` leaves them.
            let entry = out.add(n);
            let (pixel, scratch) = rc2k_core::gfx::pixels::blend_entry(
                format,
                first.add(n).read_unaligned(),
                second.add(n).read_unaligned(),
                entry.read_unaligned(),
                towards_second,
            );
            if let Some(scratch) = scratch {
                game_write(SCRATCH_RVA, scratch);
            }
            entry.write_unaligned(pixel);
        }
    }
    Ok(())
}
