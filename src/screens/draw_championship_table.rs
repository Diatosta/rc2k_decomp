//! `screens::draw_championship_table` (0x4A4E1E): the championship standings: heads, the six
//! rounds' flags and ten rows of position, name, points, total, car and flag (see
//! `screens/championship_table.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758E56). Refuses (before anything) no history table; a
//! position not in it is the original's fatal error 0x8070A7FA. Compared in shadow mode
//! (`dispatch_draw`; the row counters and the line buffer as state). Self-test: both event
//! modes, nothing drawn (skipped without the fonts or a history holding the ten rows).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::screens::championship_table::{championship_table_ready, draw_championship_table};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_championship_table",
    rva(0x4A4E1E),
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
    unsafe { draw_championship_table(canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    capture(&[
        (crate::screens::line_buffer() as u32, 0x40),
        at(rva(0x755579)),
        at(rva(0x7556BC)),
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
    if !unsafe { championship_table_ready() } {
        return t.skip("no championship history with the ten rows");
    }
    let regions: Vec<Region> = vec![at(rva(0x7556D0)), at(rva(0x755579)), at(rva(0x7556BC))];
    let saved = capture(&regions);
    let mode: u32 = unsafe { game_read(rva(0x7556D0)) };
    for m in [mode, 0x13, 0] {
        check_draw(
            &mut t,
            &format!("event mode {m:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(rva(0x7556D0), m) },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
