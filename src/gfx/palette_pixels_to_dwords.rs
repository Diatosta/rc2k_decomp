//! `gfx::palette_pixels_to_dwords` (0x41CCCB): turns the picture buffer's decoded palette
//! numbers, a byte a pixel, into the palette's dwords, in place.
//!
//! In:  nothing
//! Out: every register preserved
//! `g_pictureWidth` * `g_pictureHeight` pixels, from the last down, each through
//! `g_decodedPalette`. One caller, `gfx::read_png`, for a file with a palette (none of the
//! game's has one, so the self-test is all it gets).
//! Refuses what `pixels_to_dwords::pixel_count` refuses. The name is a reading.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::gfx::pcx::PALETTE_RVA;
use crate::gfx::picture::picture;
use crate::gfx::pixels_to_dwords::{check_spread, dispatch_spread, pixel_count};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "gfx::palette_pixels_to_dwords",
    rva(0x41CCCB),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn palette_pixels_to_dwords() -> Result<(), BadInput> {
    unsafe {
        let count = pixel_count()?;
        // Converted: `rc2k_core::gfx::pixels::palette_to_dwords`.
        let buffer = std::slice::from_raw_parts_mut(picture(), count * 4);
        let palette: [u32; 256] = game_read(PALETTE_RVA);
        rc2k_core::gfx::pixels::palette_to_dwords(buffer, count, &palette)
            .ok_or(BadInput("a picture buffer shorter than its dwords"))
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_spread(&HOOK, r, call_original, palette_pixels_to_dwords);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if picture().is_null() {
        return t.skip("the picture buffer is not allocated");
    }
    check_spread(
        &mut t,
        ORIGINAL.load(Ordering::SeqCst),
        palette_pixels_to_dwords,
    );
    t.finish()
}
