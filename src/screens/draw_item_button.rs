//! `screens::draw_item_button` (0x469283): draws the button a menu item describes.
//!
//! In:  edi = the menu item
//! Out: every register preserved
//! Width from the item's +8, alignment from +0xC, position from +0 and +4; through
//! `draw_button_highlighted` if the item is the highlighted one (`dword_755525` equals its
//! type at +0x10), else through `draw_button`. Two callers, both in `draw_menu_items`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::buttons::{PLAIN, check_item_button, draw_item_button};
use crate::screens::harness::{Left, dispatch_draw, drawing_unavailable};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::draw_item_button", rva(0x469283), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version: `buttons::draw_item_button` ----

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_item_button(r.edi, canvas) }?;
    Ok(Left::nothing())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &|| PLAIN.state());
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable() {
        return t.skip(why);
    }
    check_item_button(&mut t, call_original, &rust);
    t.finish()
}
