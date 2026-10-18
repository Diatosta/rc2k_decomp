//! `screens::draw_record_heading` (0x4A247D): the record pages' heading (the section's name or its first record's, and the record's stage line) (see `screens/best_scores.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers 0x4A2112, 0x4A2F47, `screens::draw_record_times`. Compared in shadow mode (`dispatch_draw`). Self-test: the records 0 .. 2 of the
//! page's section, nothing drawn (skipped without the fonts or the records).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::screens::best_scores::*;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_record_heading",
    rva(0x4A247D),
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
    unsafe { draw_record_heading(canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    capture(&[
        (crate::screens::line_buffer() as u32, 0x100),
        (game_addr(rva(0x7559AC)) as u32, 0xC),
        (crate::strings::text_from_ids::text() as u32, 0x100),
    ])
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
    let regions: Vec<Region> = vec![at(rva(0x755958))];
    let saved = capture(&regions);
    for record in [0u32, 1, 2] {
        unsafe { game_write(rva(0x755958), record) };
        if !unsafe { record_page_ready() } {
            continue;
        }
        check_draw(
            &mut t,
            &format!("record {record}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(rva(0x755958), record) },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
