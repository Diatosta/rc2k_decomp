//! `screens::draw_no_controller` (0x46AEAA): string 0x56D at (0x1BC, 0xD5) (see
//! `screens/controller.rs`).
//!
//! In:  nothing
//! Out: as `draw_string_by_id` leaves them (carry, eax 0, ebx, ecx, edx)
//!
//! Caller: `screens::draw_controller_page`. Compared in shadow mode (`dispatch_draw`).
//! Self-test: once, nothing drawn (skipped without the fonts).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::controller::draw_no_controller;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw, draw_entry};
use crate::service::harness::capture;
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_no_controller",
    rva(0x46AEAA),
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

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_no_controller(canvas) }
}

fn state() -> Vec<u8> {
    capture(&[(crate::strings::text_from_ids::text() as u32, 0x100)])
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_no_controller", r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    check_draw(
        &mut t,
        "once",
        call_original,
        Regs::SENTINEL,
        &|| {},
        &rust,
        &state,
    );
    t.finish()
}
