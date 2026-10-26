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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::menu::standings::*;

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const ENTERED_RVA: usize = rva(0x755920); // dword_755920
// dword_7558F4
// dword_7558F8
// g_yesNoAnswer

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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { best_overall_scores(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
