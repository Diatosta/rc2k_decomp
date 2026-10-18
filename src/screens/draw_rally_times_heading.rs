//! `screens::draw_rally_times_heading` (0x48AAF6): a rally stage times page's heading: the stage's name and its stage line (see `screens/best_scores.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: `screens::draw_rally_stage_times`, 0x489571, 0x489AE3. Refuses (before anything) no stage records. Compared in shadow mode
//! (`dispatch_draw`). Self-test: the page's own stage and the rally's stages 0 .. 2, nothing drawn (skipped
//! without the fonts or the records).

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
    "screens::draw_rally_times_heading",
    rva(0x48AAF6),
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
    unsafe { draw_times_heading(&RALLY_STAGE_TIMES, canvas) }?;
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
    let regions: Vec<Region> = vec![at(rva(0x7587F0))];
    let saved = capture(&regions);
    for record in [0u32, 1, 2] {
        unsafe { game_write(rva(0x7587F0), record) };
        if !unsafe { times_page_ready(&RALLY_STAGE_TIMES) } {
            continue;
        }
        check_draw(
            &mut t,
            &format!("record {record}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(rva(0x7587F0), record) },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
