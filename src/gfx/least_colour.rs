//! `gfx::least_colour` (0x4053BC): the least colour a texture's format has.
//!
//! In:  eax = the format (`PixelFormat`)
//! Out: eax = the components of a pixel with the lowest bit of each run of each mask
//!      set; everything else preserved
//!
//! Through `gfx::pixel_to_components` (`gfx/texture_pixels.rs`). Two call sites, as
//! `gfx::alpha_from_kept`'s: the key black is replaced with.
//! Refuses a null format.
//! Self-test: the nine test formats.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::gfx::harness::{check_pixels, dispatch_pixels};
use crate::gfx::pixel_format::{PixelFormat, TEST_FORMATS};
use crate::gfx::texture_pixels;
use crate::screens::harness::Left;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::least_colour", rva(0x4053BC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.eax == 0 {
        return Err(BadInput("a null format").into());
    }
    let format = unsafe { (r.eax as *const PixelFormat).read_unaligned() };
    Ok(Left::nothing().with(EAX, texture_pixels::least_colour(&format)))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_pixels(&HOOK, r, call_original, &[], &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let original = ORIGINAL.load(Ordering::SeqCst);
    for (case, format) in TEST_FORMATS.iter().enumerate() {
        check_pixels(
            &mut t,
            &format!("case {case}: {format:x?}"),
            original,
            Regs {
                eax: format as *const PixelFormat as u32,
                ..Regs::SENTINEL
            },
            &|| {},
            &[],
            &rust,
        );
    }
    t.finish()
}
