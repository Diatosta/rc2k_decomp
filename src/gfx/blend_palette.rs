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

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::gfx::pixel_format::{PixelFormat, SCRATCH_RVA, TEST_FORMATS, surface_format};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::blend_palette", rva(0x405F0A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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
            // Converted: `rc2k_core::pixels::blend_entry`; the scratch bytes as
            // `components_to_pixel` leaves them.
            let entry = out.add(n);
            let (pixel, scratch) = rc2k_core::pixels::blend_entry(
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

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.ebx == 0 {
        return Err(BadInput("no pixel format").into());
    }
    unsafe {
        let format = (r.ebx as *const PixelFormat).read_unaligned();
        blend_palette(
            r.esi as *const u32,
            r.edx as *const u32,
            r.edi as *mut u32,
            r.ecx,
            r.eax,
            &format,
        )?;
    }
    Ok(Left::nothing())
}

/// The result and the conversion's scratch bytes.
fn regions(r: &Regs) -> [Region; 2] {
    let len = if r.edi != 0 && (1..=0x1000).contains(&r.ecx) {
        r.ecx as usize * 4
    } else {
        0
    };
    [(r.edi, len), at(SCRATCH_RVA)]
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test: palettes of its own ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let scratch = [at(SCRATCH_RVA)];
    let saved = capture(&scratch);
    let first: Vec<u32> = (0..15u32)
        .map(|n| n.wrapping_mul(0x0123_4567) ^ 0x00F0_3C1E)
        .collect();
    let second: Vec<u32> = (0..15u32)
        .map(|n| n.wrapping_mul(0x89AB_CDEF) ^ 0xFF12_8040)
        .collect();
    let mut out = vec![0u32; 15];
    for format in TEST_FORMATS.iter().chain([&surface_format()]) {
        for towards_second in [0u32, 1, 0x80, 0xFF, 0x100, 0xFFFF_FFFF] {
            for count in [1u32, 15] {
                let before = Regs {
                    eax: towards_second,
                    ebx: format as *const PixelFormat as u32,
                    ecx: count,
                    edx: second.as_ptr() as u32,
                    esi: first.as_ptr() as u32,
                    edi: out.as_mut_ptr() as u32,
                    ..Regs::SENTINEL
                };
                let regions = regions(&before);
                check_state(
                    &mut t,
                    &format!("{format:x?} towards_second={towards_second:#x} count={count}"),
                    call_original,
                    before,
                    &|| restore(&regions, &[0xAA; 15 * 4 + 4][..count as usize * 4 + 4]),
                    &regions,
                    &rust,
                );
            }
        }
    }
    restore(&scratch, &saved);
    t.finish()
}
