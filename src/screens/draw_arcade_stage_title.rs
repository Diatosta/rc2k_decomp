//! `screens::draw_arcade_stage_title` (0x488F2F): the arcade's after-stage screen's title:
//! string 0x1FC at (0x19A, 0x118) with the font at 0x51E8A4. In that screen's draw list
//! (0x758E66). IDA had it as code outside any function (made one 2026-10-19).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the carry as `text::draw_string_by_id`
//!      leaves it
//!
//! Compared in shadow mode (`dispatch_draw`). Self-test: the game's string and font,
//! nothing drawn.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::font_at;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::capture;
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_arcade_stage_title",
    rva(0x488F2F),
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

const FONT: usize = 0x51E8A4;
const TITLE: u32 = 0x1FC;

// ---- Rust version ----

pub unsafe fn draw_arcade_stage_title(canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    let out = unsafe { draw_string_by_id_at(TITLE, font_at(FONT), 0x19A, 0x118, canvas) }?;
    Ok(Left::carry(out.too_wide))
}

// ---- hook ----

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_arcade_stage_title(canvas) }
}

fn state() -> Vec<u8> {
    capture(&[(text() as u32, TEXT_LEN)])
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    let saved = state();
    check_draw(
        &mut t,
        "the title",
        call_original,
        Regs::SENTINEL,
        &|| crate::service::harness::restore(&[(text() as u32, TEXT_LEN)], &saved),
        &rust,
        &state,
    );
    t.finish()
}
