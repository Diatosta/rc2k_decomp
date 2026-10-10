//! `gfx::pixel_to_components` (0x405CFA): takes a pixel apart into four 8-bit components,
//! one per byte of the result.
//!
//! In:  eax = pixel, ebx = pixel format
//! Out: eax = the components; everything else preserved
//! A pixel of 0 gives 0 without looking at the format. With a format that has rotation
//! counts, each component is the pixel's masked bits rotated right by its count. With the
//! other kind, each component's bits are picked out of the pixel one at a time from the top
//! bit down, at most eight of them, and left-aligned in the byte if there are fewer.
//! Six callers. Manual patch with a private copy (0xBA bytes, nothing to relocate): a
//! conditional jump sits in the first bytes.
//! Refuses a null format.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::gfx::pixel_format::{PixelFormat, TEST_FORMATS, TEST_VALUES, surface_format};
use crate::screens::harness::Left;
use crate::service::harness::check_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::pixel_to_components", rva(0x405CFA), thunk, &ORIGINAL)
        .private_copy(0xBA)
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub fn pixel_to_components(format: &PixelFormat, pixel: u32) -> u32 {
    if pixel == 0 {
        return 0;
    }
    if let Some(rotations) = format.rotations() {
        return (0..4).fold(0, |all, n| {
            all | (pixel & format.masks[n]).rotate_right(rotations[n])
        });
    }
    let mut components = [0u8; 4];
    for (n, component) in components.iter_mut().enumerate() {
        // A 1 marks how far the byte has been filled: when it falls out, there are eight.
        let mut bits: u8 = 1;
        let mut full = false;
        let mut bit = format.layout as i32;
        while bit >= 0 && !full {
            let at = bit as u32 & 31;
            if format.masks[n] >> at & 1 != 0 {
                full = bits & 0x80 != 0;
                bits = bits << 1 | (pixel >> at & 1) as u8;
            }
            bit -= 1;
        }
        while !full {
            full = bits & 0x80 != 0;
            bits <<= 1;
        }
        *component = bits;
    }
    u32::from_le_bytes(components)
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.ebx == 0 {
        return Err(BadInput("no pixel format").into());
    }
    let format = unsafe { (r.ebx as *const PixelFormat).read_unaligned() };
    Ok(Left::nothing().with(EAX, pixel_to_components(&format, r.eax)))
}

// Written out instead of going through `dispatch_state`, which allocates: this is called per
// pixel.
extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if r.ebx == 0 {
        unsafe { call_original(r) };
        return;
    }
    let format = unsafe { (r.ebx as *const PixelFormat).read_unaligned() };
    let mine = pixel_to_components(&format, r.eax);
    if !shadow() {
        r.eax = mine;
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.eax != mine || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| {
            format!(
                "{format:x?} pixel={:#x}: original {:#x}; rust {mine:#x}",
                r.eax, o.eax
            )
        });
    }
    r.eax = o.eax;
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for format in TEST_FORMATS.iter().chain([&surface_format()]) {
        for pixel in TEST_VALUES {
            let before = Regs {
                eax: pixel,
                ebx: format as *const PixelFormat as u32,
                ..Regs::SENTINEL
            };
            check_state(
                &mut t,
                &format!("{format:x?} pixel={pixel:#x}"),
                call_original,
                before,
                &|| {},
                &[],
                &rust,
            );
        }
    }
    t.finish()
}
