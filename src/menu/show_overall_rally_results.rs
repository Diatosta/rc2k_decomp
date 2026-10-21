//! `menu::show_overall_rally_results` (0x4A3BA2): the overall rally results: the first player's entry of any group in standings
//! table A (`menu::first_player_a`), its figure held to 0x51 when above 0xAB (the original's: section 16) into `dword_7556B4`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::overall_rally_results_loop`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_4A0E60` (twice), `sub_4A94EC`. The steps are their Rust versions; the page through `part` (1), in
//! the game its Rust version (`screen_loop::run_screen`). Ungated (the page waits for the
//! player). Refuses what the steps refuse, before anything; raises the search's fatal
//! error as the original. Self-test: a copy whose page call goes to a stand-in, against
//! the Rust version, on a standings table of the test's.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::menu::standings::*;
use crate::service::harness::{Region, at, capture, restore};
use rc2k_core::menu::results_pages::{Page, held_figure};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::show_overall_rally_results",
    rva(0x4A3BA2),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const CLASS_RVA: usize = rva(0x7558F4); // dword_7558F4
const COUNT_RVA: usize = rva(0x7556B4); // dword_7556B4
const EVENT_MODE_RVA: usize = rva(0x7556D0); // g_eventMode

/// A call by place: 1 the page.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn show_overall_rally_results(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        g.edi = game_read(TABLE_A_RVA);
        g.ebx = ANY_GROUP;
        let found = crate::menu::first_player_a::first_player_a(&g)?;
        let count = found.edx;
        // Converted: `rc2k_core::menu::results_pages::held_figure`.
        let count = held_figure(Page::OverallRally, count);
        game_write(COUNT_RVA, count);
        let g = crate::menu::first_page_stage::first_page_stage(&found)?;
        let mut r = crate::menu::stage_range_caption::stage_range_caption(&g)?;
        part(1, &mut r)?;
    }
    Ok(*entry)
}

/// In the game: the page's Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        crate::menu::screen_loop::run_screen(&crate::menu::overall_rally_results_loop::SCREEN, r)
    };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { show_overall_rally_results(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let entry: &[u8] = &[0x60, 0xBB, 0x63, 0x00, 0x00, 0x00];
    let copy = match copy(
        0x4A3BA2,
        0x3E,
        entry,
        &[0xC, 0x2D, 0x32, 0x37],
        &[(0x466B21, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut buf = vec![0u32; (9 * A_ENTRY / 4) as usize];
    let tp = buf.as_mut_ptr() as u32;
    let regions: Vec<Region> = vec![
        at(CLASS_RVA),
        at(COUNT_RVA),
        at(rva(0x7556B8)),
        (game_addr(rva(0xA8BE54)) as u32, 0x20),
    ];
    let mut vars = regions.clone();
    vars.extend([TABLE_A_RVA, EVENT_MODE_RVA].map(at));
    let saved = capture(&vars);
    for case in 0..4u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(TABLE_A_RVA, tp);
                fill_test_table(tp, A_ENTRY, 8, 0x1C, 0);
                // a player in group {case} % 3 + 1, figure by lot
                put(tp + A_ENTRY * 4 + 4, 2);
                put(tp + A_ENTRY * 4 + 0x1C, case % 3 + 1);
                put(
                    tp + A_ENTRY * 4 + 0x44,
                    [3u32, 0x40, 0xC0, 0x1FF][case as usize],
                );
                game_write(EVENT_MODE_RVA, if case == 2 { 0x13u32 } else { 0 });
            },
            &|r| unsafe { show_overall_rally_results(&Regs { ebx: r.ebx, ..*r }, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    drop(buf);
    t.finish()
}
