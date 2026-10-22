//! `screens::draw_item_button_centred_highlighted` (0x46923F): draws a highlighted, centred button as wide as a menu item's text.
//!
//! In:  edi = the menu item; ecx is passed on to the measuring as its x
//! Out: every register preserved
//! The item's string (+0x14) is measured with the item's font at +0x4C
//! (`text::measure_string_by_id`, which leaves the text in the shared text buffer); the
//! button is that wide plus 12, at the item's +0 and +4, through `draw_button_highlighted`.
//! One caller, `draw_menu_items`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::buttons::{ALIGN_CENTRE, HIGHLIGHTED, check_text_button, draw_text_button};
use crate::screens::harness::{Left, dispatch_draw, draw_entry, drawing_unavailable};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_text::Canvas;
use crate::text::measure_string_by_id::measuring_unavailable;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_item_button_centred_highlighted",
    rva(0x46923F),
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

// ---- Rust version: `buttons::draw_text_button` ----

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_text_button(&HIGHLIGHTED, ALIGN_CENTRE, r.edi, r.ecx as i32, canvas) }?;
    Ok(Left::nothing())
}

// ---- hook ----

fn state() -> Vec<u8> {
    [
        HIGHLIGHTED.state(),
        unsafe { from_raw_parts(text(), TEXT_LEN) }.to_vec(),
    ]
    .concat()
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_item_button_centred_highlighted", r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable().or_else(measuring_unavailable) {
        return t.skip(why);
    }
    check_text_button(&mut t, &HIGHLIGHTED, call_original, &rust);
    t.finish()
}
