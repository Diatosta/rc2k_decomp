//! `menu::rally_results` (0x4A94EC): the rally's results pages, `dword_75AADC` 1 while
//! they show (the pages' button "voltar"). Outside event mode 0x13: standings table A from
//! all (`menu::standings_a_from_all`), sorted by total (`menu::sort_standings_by_total`),
//! cleared and numbered (`menu::clear_standings_a`, `menu::number_all_a`), the overall
//! page (`menu::show_overall_rally_results`); in it: table A from all and the class pages
//! (`menu::class_rally_results_pass`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126`. The steps are their Rust versions; the pages through `part`, in
//! the game their Rust versions. Ungated (the pages wait for the player). Refuses what the
//! steps refuse. Self-test: a copy whose page calls go to stand-ins, against the Rust
//! version, in both modes, on the game's tables; put back.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::menu::standings::*;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::rally_results", rva(0x4A94EC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const FLAG_RVA: usize = rva(0x75AADC); // dword_75AADC
const EVENT_MODE_RVA: usize = rva(0x7556D0); // g_eventMode

/// A call by place: 1 `menu::show_overall_rally_results`, 2 `menu::class_rally_results_pass`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn rally_results(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        game_write(FLAG_RVA, 1u32);
        g = crate::menu::standings_a_from_all::standings_a_from_all(&g)?;
        // Converted: `rc2k_core::menu::results_pages::class_pages_only`.
        if !rc2k_core::menu::results_pages::class_pages_only(game_read(EVENT_MODE_RVA)) {
            g = crate::menu::sort_standings_by_total::sort_standings_by_total(&g)?;
            g = crate::menu::clear_standings_a::clear_standings_a(&g)?;
            g = crate::menu::number_all_a::number_all_a(&g)?;
            part(1, &mut g)?;
        } else {
            part(2, &mut g)?;
        }
        game_write(FLAG_RVA, 0u32);
    }
    Ok(*entry)
}

/// In the game: the pages' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        *r = match place {
            1 => {
                use crate::menu::show_overall_rally_results as p;
                p::show_overall_rally_results(r, &mut p::game_part)
            }
            _ => {
                use crate::menu::class_rally_results_pass as p;
                p::class_rally_results_pass(r, &mut p::game_part)
            }
        }?;
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { rally_results(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x4A94EC,
        0x45,
        &[
            0x60, 0xC7, 0x05, 0xDC, 0xAA, 0x75, 0x00, 0x01, 0x00, 0x00, 0x00,
        ],
        &[0x14, 0x19, 0x1E, 0x23, 0x28, 0x2F, 0x34],
        &[(0x4A3BA2, 1), (0x4A32A5, 2)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let table: u32 = unsafe { game_read(TABLE_A_RVA) };
    if table == 0 || (0..4).all(|n| player(n) == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut regions: Vec<Region> = vec![
        (table, (200 * A_ENTRY) as usize),
        (game_addr(KEPT_NAME_RVA) as u32, 0x1F),
        at(rva(0x7559B4)),
        at(rva(0x7559B8)),
        at(FLAG_RVA),
    ];
    regions.extend(
        (0..4u32)
            .map(|n| (player(n), 0x400))
            .filter(|&(p, _)| p != 0),
    );
    let mut vars = regions.clone();
    // `dword_75597C` was left changed (2026-10-18): put back too.
    vars.extend([EVENT_MODE_RVA, PLAYER_COUNT_RVA, rva(0x75597C)].map(at));
    let saved = capture(&vars);
    for case in 0..2u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(EVENT_MODE_RVA, if case == 1 { 0x13u32 } else { 0 });
            },
            &|r| unsafe { rally_results(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
