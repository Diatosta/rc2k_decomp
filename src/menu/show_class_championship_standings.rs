//! `menu::show_class_championship_standings` (0x4A4AD5): the championship standings of a class: `dword_7558F4` the class (ebx); the first player's entry of class ebx in standings
//! table B (`menu::first_player_b`), its figure held to 0x33 into `dword_7556B4`;
//! `menu::first_page_stage`, `menu::stage_range_caption`; then the page
//! (`menu::class_championship_standings_loop`).
//!
//! In:  ebx the class
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A4A1C`. The steps are their Rust versions; the page through `part` (1), in
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
    "menu::show_class_championship_standings",
    rva(0x4A4AD5),
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
pub unsafe fn show_class_championship_standings(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        g.eax = game_read(rva(0x7559AC)); // seconds
        game_write(CLASS_RVA, g.ebx);
        g.edi = game_read(TABLE_B_RVA);
        let found = crate::menu::first_player_b::first_player_b(&g)?;
        let count = found.edx;
        // Converted: `rc2k_core::menu::results_pages::held_figure`.
        let count = held_figure(Page::ClassChampionship, count);
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
        crate::menu::screen_loop::run_screen(
            &crate::menu::class_championship_standings_loop::SCREEN,
            r,
        )
    };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { show_class_championship_standings(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { show_class_championship_standings(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both_from, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let entry: &[u8] = &[0x60, 0xA1, 0xAC, 0x59, 0x75, 0x00];
    let copy = match copy(
        0x4A4AD5,
        0x41,
        entry,
        &[0x12, 0x30, 0x35, 0x3A],
        &[(0x4A4B6B, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut buf = vec![0u32; (9 * B_ENTRY / 4) as usize];
    let tp = buf.as_mut_ptr() as u32;
    let regions: Vec<Region> = vec![
        at(CLASS_RVA),
        at(COUNT_RVA),
        at(rva(0x7556B8)),
        (game_addr(rva(0xA8BE54)) as u32, 0x20),
    ];
    let mut vars = regions.clone();
    vars.extend([TABLE_B_RVA, EVENT_MODE_RVA].map(at));
    let saved = capture(&vars);
    for case in 0..4u32 {
        run_both_from(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(TABLE_B_RVA, tp);
                fill_test_table(tp, B_ENTRY, 8, 0x10, 0);
                // a player in group {case} % 3 + 1, figure by lot
                put(tp + B_ENTRY * 4 + 4, 2);
                put(tp + B_ENTRY * 4 + 0x10, case % 3 + 1);
                put(
                    tp + B_ENTRY * 4 + 0x70,
                    [3u32, 0x40, 0xC0, 0x1FF][case as usize],
                );
                game_write(EVENT_MODE_RVA, if case == 2 { 0x13u32 } else { 0 });
            },
            Regs {
                ebx: case % 3 + 1,
                ..Regs::SENTINEL
            },
            &|r| unsafe { show_class_championship_standings(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    drop(buf);
    t.finish()
}
