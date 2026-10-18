//! `screens::draw_overall_best_scores` (0x4A8FA2): the overall best scores (group 4) with its heading
//! (the section's name in event mode 0x13, else string 0x362), the player's own scores in
//! the highlight fonts (see `screens/best_scores.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758D42). Refuses (before anything) no table.
//! Compared in shadow mode (`dispatch_draw`). Self-test: event modes 0 and 0x13, nothing
//! drawn (skipped without the fonts or the tables).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::screens::best_scores::*;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_overall_best_scores",
    rva(0x4A8FA2),
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
    unsafe { draw_scores_with_heading(&OVERALL_BEST_SCORES, false, canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    capture(&[
        (crate::screens::line_buffer() as u32, 0x20),
        (game_addr(rva(0x7559AC)) as u32, 0x44),
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
    if unsafe { game_read::<u32>(rva(0x7556D4)) } == 0
        || unsafe { game_read::<u32>(rva(0x7556DC)) } == 0
    {
        return t.skip("no best scores tables");
    }
    let regions: [Region; 3] = [
        at(rva(0x7556D0)),
        (crate::screens::line_buffer() as u32, 0x20),
        (game_addr(rva(0x7559AC)) as u32, 0x44),
    ];
    let saved = capture(&regions);
    for page in [0u32, 0x13] {
        check_draw(
            &mut t,
            &format!("mode {page:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(rva(0x7556D0), page) },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
