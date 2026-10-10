//! `screens::draw_button_alt` (0x468EDD): draws a menu button with its plain end pieces, like `draw_button`
//! without its slips and with work variables of its own.
//!
//! In:  eax = width of the middle, ebx = x, ecx = y, edx = alignment (1 centre, 2 left,
//!      3 right), edi = the menu item, read for its sprites (+0x20)
//! Out: every register preserved
//! One caller: `draw_item_button_right`. Fatal errors 0x80702023 and 0x80701010.
//! The drawing itself, the alignments and the two fatal errors are in `screens/buttons.rs`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::buttons::{ALT, check_kind, draw_button};
use crate::screens::harness::{Left, dispatch_draw, drawing_unavailable};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::draw_button_alt", rva(0x468EDD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version: `buttons::draw_button` with `ALT` ----

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe {
        draw_button(
            &ALT,
            r.eax,
            r.ebx as i32,
            r.ecx as i32,
            r.edx,
            r.edi,
            canvas,
        )
    }?;
    Ok(Left::nothing())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &|| ALT.state());
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable() {
        return t.skip(why);
    }
    check_kind(&mut t, &ALT, call_original, &rust);
    t.finish()
}
