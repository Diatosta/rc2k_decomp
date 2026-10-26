//! `menu::best_class_scores` (0x4A8608): each class's (0 .. `dword_7558F8`) new best scores: each counted player of the class whose score (+0xB0) is not below the class's tenth is entered (`menu::enter_best_score`); with one entered, `dword_7558F4` the class and the page (`menu::best_class_scores_loop`), stopping when the leaving is answered yes (`g_yesNoAnswer` 1).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: the event drivers. The entries are their Rust versions; the page through `part` (1), in
//! the game its Rust version (`screen_loop::run_screen`). Ungated (the page waits for the
//! player). Refuses a player count of 0 or past 4, a missing player, no classes or more than five, no table, and what an entry refuses, before anything. Self-test: a copy whose page call goes to a stand-in (the entries'
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
const CLASS_RVA: usize = rva(0x7558F4); // dword_7558F4
const CLASSES_RVA: usize = rva(0x7558F8); // dword_7558F8
const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer

/// A call by place: 1 the page.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn best_class_scores(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe {
        let players = counted_players()?;
        let classes: u32 = game_read(CLASSES_RVA);
        let table: u32 = game_read(BEST_SCORES_RVA);
        if classes == 0 || classes > 5 || table == 0 {
            return Err(BadInput("no classes, more than five, or no best-score table").into());
        }
        let mut g = *entry;
        for class in 0..classes {
            g.edx = class;
            game_write(ENTERED_RVA, 0u32);
            for (n, &p) in players.iter().enumerate() {
                game_write(CURRENT_RVA, n as u32);
                g.eax = get(p + 0xB0);
                g.ebx = get(p + 0x90);
                if g.ebx == class {
                    g.ebx = class * 0x258 + 0x21C;
                    g.esi = table + g.ebx;
                    // Converted: `rc2k_core::menu::bests::makes_score`.
                    if rc2k_core::menu::bests::makes_score(g.eax, get(g.esi + 0x38)) {
                        enter_best(None, 0xB0)?;
                        game_write(ENTERED_RVA, 1u32);
                    }
                }
            }
            g.ecx = players.len() as u32;
            if game_read::<u32>(ENTERED_RVA) == 1 {
                game_write(CLASS_RVA, class);
                let mut r = g;
                part(1, &mut r)?;
                if game_read::<u32>(ANSWER_RVA) == 1 {
                    break;
                }
            }
        }
    }
    Ok(*entry)
}

/// In the game: the page's Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        crate::menu::screen_loop::run_screen(&crate::menu::best_class_scores_loop::SCREEN, r)
    };
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { best_class_scores(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
