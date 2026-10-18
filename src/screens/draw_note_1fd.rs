//! `screens::draw_note_1fd` (0x4881F8): string 0x1FD drawn at (0x19A, 0x118) in the font at
//! 0x51E8A4 (`text::draw_string_by_id`).
//!
//! In:  nothing
//! Out: as `draw_string_by_id` leaves them
//!
//! In a menu loop's draw list (0x758C62). Compared in shadow mode (`dispatch_draw`).
//! Self-test: once, nothing drawn (skipped without the fonts).

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

pub static HOOK: RawHook =
    RawHook::new("screens::draw_note_1fd", rva(0x4881F8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the outcome's registers.
pub unsafe fn draw_note_1fd(canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    use crate::framework::{EAX, EBX, ECX, EDX};
    unsafe {
        let font = font_at(0x51E8A4);
        let out = draw_string_by_id_at(0x1FD, font, 0x19A, 0x118, canvas)?;
        Ok(Left::carry(out.too_wide)
            .with(EAX, 0)
            .with(EBX, out.text as u32)
            .with(ECX, out.x as u32)
            .with(EDX, out.y as u32))
    }
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_note_1fd(canvas) }
}

fn state() -> Vec<u8> {
    capture(&[(text() as u32, TEXT_LEN)])
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
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
