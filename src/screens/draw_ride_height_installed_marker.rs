//! `screens::draw_ride_height_installed_marker` (0x4A6EB7): draws the second marker (glyph 0x77) of the lower left setup bar, at the
//! percentage in `dword_75ACAC`; always drawn.
//!
//! In:  ebx, passed on as the palette at 0 percent or less
//! Out: eax = the percentage; everything else preserved
//! The conditions, the bars and how the screen was confirmed are in `setup_markers.rs`.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::drawing_unavailable;
use crate::screens::setup_markers::{
    LEFT_X, LOWER_LEFT_INSTALLED_RVA, Marker, check_marker, dispatch_marker,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_ride_height_installed_marker",
    rva(0x4A6EB7),
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

// ---- Rust version: `setup_markers::draw_marker` with this ----

pub const MARKER: Marker = Marker::Always {
    x: LEFT_X,
    percent_rva: LOWER_LEFT_INSTALLED_RVA,
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_marker(&HOOK, r, call_original, MARKER);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable() {
        return t.skip(why);
    }
    check_marker(&mut t, call_original, MARKER);
    t.finish()
}
