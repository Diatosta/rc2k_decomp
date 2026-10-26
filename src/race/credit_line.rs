//! `race::credit_line` (0x4161BA): string eax in the font at ebx at (ecx, edx), unless
//! edx is at or below -0x1B or at or past the screen's bottom (see `race/frame_text.rs`).
//!
//! In:  eax = string id, ebx = font, ecx = x, edx = y
//! Out: as `text::draw_string_by_id` leaves them, or all kept
//!
//! Callers: `race::roll_credits` (eight places). Shadow mode: the drawing harness.
//! Self-test: a string in the credits' font at heights above, on and below the screen
//! (skipped without the blitter's recording or the font).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::frame_text::credit_line_at;
use crate::screens::font_is_loaded;
use crate::screens::harness::{Left, check_draw, dispatch_draw, draw_entry, drawing_unavailable};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::credit_line", rva(0x4161BA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { credit_line_at(r, canvas) }
}

fn state() -> Vec<u8> {
    unsafe { std::slice::from_raw_parts(crate::strings::text_from_ids::text(), 0x40).to_vec() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("race::credit_line", r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable() {
        return t.skip(why);
    }
    if !font_is_loaded(0x51E6A4) {
        return t.skip("the font is not loaded");
    }
    let text = crate::strings::text_from_ids::text();
    let saved = unsafe { std::slice::from_raw_parts(text, 0x40).to_vec() };
    let font = game_addr(rva(0x51E6A4)) as u32;
    for y in [-0x1Bi32, -0x1A, 100, 479, 480, 0x7FFF_FFFF] {
        check_draw(
            &mut t,
            &format!("y {y}"),
            call_original,
            Regs {
                eax: 0x10,
                ebx: font,
                ecx: 200,
                edx: y as u32,
                ..Regs::SENTINEL
            },
            &|| unsafe { std::ptr::copy_nonoverlapping(saved.as_ptr(), text, 0x40) },
            &rust,
            &state,
        );
    }
    unsafe { std::ptr::copy_nonoverlapping(saved.as_ptr(), text, 0x40) };
    t.finish()
}
