//! `screens::draw_ride_height_previous_marker` (0x4A6F39): draws the marker of the lower left setup bar (menu item 0x12), at the percentage in
//! `dword_75ACA8`.
//!
//! In:  ebx, passed on as the palette at 0 percent or less
//! Out: eax = the percentage, or 0x12 if nothing was drawn; esi = the address of the item's
//!      repair flag if that was looked at; everything else preserved
//! The conditions, the bars and how the screen was confirmed are in `setup_markers.rs`.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::drawing_unavailable;
use crate::screens::setup_markers::{
    LEFT_X, LOWER_LEFT_EDITED_RVA, LOWER_LEFT_SHOWN_RVA, Marker, check_marker, dispatch_marker,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_ride_height_previous_marker",
    rva(0x4A6F39),
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

pub const MARKER: Marker = Marker::Lower {
    item: 0x12,
    x: LEFT_X,
    shown_rva: LOWER_LEFT_SHOWN_RVA,
    edited_rva: LOWER_LEFT_EDITED_RVA,
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
