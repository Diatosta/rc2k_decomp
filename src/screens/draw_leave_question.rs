//! `screens::draw_leave_question` (0x46FA75): the "leave?" question's text.
//!
//! In:  nothing
//! Out: as `text::draw_string_by_id` leaves them (carry, eax = 0, ebx, ecx, edx)
//!
//! String 0x537 at (0x197, 0x122) with the font at 0x51E8A4. In the question screen's
//! draw list (0x758CA2).
//! Self-test: the game's string and font, nothing drawn.

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
    "screens::draw_leave_question",
    rva(0x46FA75),
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
const QUESTION: u32 = 0x537;

// ---- Rust version ----

pub unsafe fn draw_leave_question(canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    let out = unsafe { draw_string_by_id_at(QUESTION, font_at(FONT), 0x197, 0x122, canvas) }?;
    Ok(Left::of_text(&out))
}

// ---- hook ----

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_leave_question(canvas) }
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
        "the question",
        call_original,
        Regs::SENTINEL,
        &|| crate::service::harness::restore(&[(text() as u32, TEXT_LEN)], &saved),
        &rust,
        &state,
    );
    t.finish()
}
