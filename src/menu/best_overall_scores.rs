//! `menu::best_overall_scores` (0x4A91BF): the overall new best scores: each counted player whose score (+0xCC) is not below group 4's tenth is entered (`menu::enter_best_score_4`); with one entered, the page (`menu::best_overall_scores_loop`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A0E60`. The entries are their Rust versions; the page through `part` (1), in
//! the game its Rust version (`screen_loop::run_screen`). Ungated (the page waits for the
//! player). Refuses a player count of 0 or past 4, a missing player, no table, and what an entry refuses, before anything. Self-test: a copy whose page call goes to a stand-in (the entries'
//! to their hooks), against the Rust version, on the game's tables with scores by lot;
//! put back.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::menu::standings::*;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::best_overall_scores", rva(0x4A91BF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const ENTERED_RVA: usize = rva(0x755920); // dword_755920
const CLASS_RVA: usize = rva(0x7558F4); // dword_7558F4
const CLASSES_RVA: usize = rva(0x7558F8); // dword_7558F8
const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer

/// A call by place: 1 the page.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn best_overall_scores(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe {
        let players = counted_players()?;
        let table: u32 = game_read(BEST_SCORES_RVA);
        if table == 0 {
            return Err(BadInput("no best-score table").into());
        }
        let mut g = *entry;
        game_write(ENTERED_RVA, 0u32);
        for (n, &p) in players.iter().enumerate() {
            game_write(CURRENT_RVA, n as u32);
            g.eax = get(p + 0xCC);
            g.ebx = 4 * 0x258 + 0x21C;
            g.esi = table + g.ebx;
            // Converted: `rc2k_core::menu::bests::makes_score`.
            if rc2k_core::menu::bests::makes_score(g.eax, get(g.esi + 0x38)) {
                enter_best(Some(4), 0xCC)?;
                game_write(ENTERED_RVA, 1u32);
            }
        }
        g.ecx = players.len() as u32;
        if game_read::<u32>(ENTERED_RVA) == 1 {
            let mut r = g;
            part(1, &mut r)?;
        }
    }
    Ok(*entry)
}

/// In the game: the page's Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        crate::menu::screen_loop::run_screen(&crate::menu::best_overall_scores_loop::SCREEN, r)
    };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { best_overall_scores(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { best_overall_scores(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x4A91BF,
        0x70,
        &[0x60, 0xB9, 0x00, 0x00, 0x00, 0x00],
        &[0x48, 0x69],
        &[(0x4A8B15, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut regions: Vec<Region> = rank_regions();
    regions.extend(
        (0..4u32)
            .map(|n| (player(n), 0x400))
            .filter(|&(p, _)| p != 0),
    );
    regions.extend([CURRENT_RVA, ENTERED_RVA, CLASS_RVA].map(at));
    if regions[0].0 == 0 || (0..4).any(|n| player(n) == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend([CLASSES_RVA, PLAYER_COUNT_RVA, ANSWER_RVA].map(at));
    let saved = capture(&vars);
    for case in 0..3u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(CLASSES_RVA, 3u32);
                game_write(PLAYER_COUNT_RVA, 1 + case);
                game_write(ANSWER_RVA, u32::from(case == 2));
                for n in 0..4u32 {
                    put(player(n) + 0x90, (n + case) % 3);
                    put(player(n) + 0xB0, [0u32, 0x7FFF_FFFF, 5][case as usize] + n);
                    put(player(n) + 0xCC, [0u32, 0x7FFF_FFFF, 5][case as usize] + n);
                }
            },
            &|r| unsafe { best_overall_scores(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
