//! `gfx::rgb_pixels_to_dwords` (0x41CC96): turns the picture buffer's decoded pixels of three
//! bytes into dwords, in place.
//!
//! In:  nothing
//! Out: every register preserved
//! `g_pictureWidth` * `g_pictureHeight` pixels, from the last down: the first byte shifted
//! up 16, the second up 8, the third; the top byte 0. One caller, `gfx::read_png`, for a
//! file without a palette: every PNG file of the game.
//! Refuses what `pixels_to_dwords::pixel_count` refuses. The name is a reading.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::gfx::picture::picture;
use crate::gfx::pixels_to_dwords::{check_spread, dispatch_spread, pixel_count};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::rgb_pixels_to_dwords", rva(0x41CC96), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn rgb_pixels_to_dwords() -> Result<(), BadInput> {
    // Converted: `rc2k_core::gfx::pixels::rgb_to_dwords`.
    let count = unsafe { pixel_count() }?;
    let buffer = unsafe { std::slice::from_raw_parts_mut(picture(), count * 4) };
    rc2k_core::gfx::pixels::rgb_to_dwords(buffer, count).expect("the slice holds the dwords");
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_spread(&HOOK, r, call_original, rgb_pixels_to_dwords);
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
        rgb_pixels_to_dwords,
    );
    t.finish()
}
