//! `screens::draw_stage_results` (0x48E740): a time trial stage's results: the menu's items,
//! the stage line, ten rows of name, time (with the gap to the first) or seal code, rank, car
//! and flag, and the fastest line (see `screens/stage_results.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758D06). Refuses (before anything) no current menu, stage
//! records or times table. Compared in shadow mode (`dispatch_draw`; the menu's items and
//! the work variables as state). Self-test: times and seal modes, the two switches each way,
//! nothing drawn (skipped without the fonts or the page's data).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::screens::stage_results::{draw_stage_results, stage_results_ready};
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_text::Canvas;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_stage_results",
    rva(0x48E740),
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
    unsafe { draw_stage_results(canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let mut regions: Vec<Region> = vec![
        (crate::screens::line_buffer() as u32, 0x100),
        g(0xA8BC54, 0x504),
        g(0x7559AC, 0x48),
        g(0x75AB24, 4),
        g(0x4E053C, 0x20),
        (crate::strings::text_from_ids::text() as u32, 0x100),
    ];
    let menu: u32 = unsafe { game_read(rva(0x75558D)) };
    if menu != 0 {
        regions.push((menu + 4, 0x5C * 4 + 0x18));
    }
    capture(&regions)
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
    let regions: Vec<Region> = vec![at(rva(0x75AB30)), at(rva(0x7556F8)), at(rva(0x7556FC))];
    let saved = capture(&regions);
    let menu: u32 = unsafe { game_read(rva(0x75558D)) };
    let items: Vec<Region> = if menu != 0 {
        vec![(menu + 4, 0x5C * 4 + 0x18)]
    } else {
        vec![]
    };
    let saved_items = capture(&items);
    for (seal, a, b) in [
        (0u32, 0u32, 0u32),
        (1, 0, 0),
        (0, 1, 0),
        (0, 0, 1),
        (1, 1, 1),
    ] {
        unsafe {
            game_write(rva(0x75AB30), seal);
            game_write(rva(0x7556F8), a);
            game_write(rva(0x7556FC), b);
        }
        if !unsafe { stage_results_ready() } {
            continue;
        }
        check_draw(
            &mut t,
            &format!("seal {seal} switches {a} {b}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(rva(0x75AB30), seal);
                game_write(rva(0x7556F8), a);
                game_write(rva(0x7556FC), b);
            },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    restore(&items, &saved_items);
    t.finish()
}
