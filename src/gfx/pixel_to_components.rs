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
    // Converted: `rc2k_core::gfx::pixels::unpack`.
    rc2k_core::gfx::pixels::unpack(format, pixel)
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
