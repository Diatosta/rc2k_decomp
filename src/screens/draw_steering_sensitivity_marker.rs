//! `screens::draw_steering_sensitivity_marker` (0x4A7027): draws the marker of the upper left setup bar (menu item 0x10), at the percentage in
//! the player's field +0x11C.
//!
//! In:  edi = player; ebx, passed on as the palette at 0 percent or less
//! Out: eax = the percentage, or 0x10 if nothing was drawn; everything else preserved
//! The conditions, the bars and how the screen was confirmed are in `setup_markers.rs`.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::drawing_unavailable;
use crate::screens::setup_markers::{LEFT_X, Marker, check_marker, dispatch_marker};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_steering_sensitivity_marker",
    rva(0x4A7027),
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

pub const MARKER: Marker = Marker::Upper {
    item: 0x10,
    x: LEFT_X,
    car_offset: 0x11C,
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
