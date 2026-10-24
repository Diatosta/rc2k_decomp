//! `menu::best_class_scores` (0x4A8608): each class's (0 .. `dword_7558F8`) new best scores: each counted player of the class whose score (+0xB0) is not below the class's tenth is entered (`menu::enter_best_score`); with one entered, `dword_7558F4` the class and the page (`menu::best_class_scores_loop`), stopping when the leaving is answered yes (`g_yesNoAnswer` 1).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: the event drivers. The entries are their Rust versions. Refuses a player count of 0 or
//! past 4, a missing player, no classes or more than five, no table, and what an entry refuses,
//! before anything.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{Carried, run_screen};
use crate::menu::standings::*;

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const ENTERED_RVA: usize = rva(0x755920); // dword_755920
const CLASS_RVA: usize = rva(0x7558F4); // dword_7558F4
const CLASSES_RVA: usize = rva(0x7558F8); // dword_7558F8
const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn best_class_scores(c: &Carried) -> Result<(), Refused> {
    unsafe {
        let players = counted_players()?;
        let classes: u32 = game_read(CLASSES_RVA);
        let table: u32 = game_read(BEST_SCORES_RVA);
        if classes == 0 || classes > 5 || table == 0 {
            return Err(BadInput("no classes, more than five, or no best-score table").into());
        }
        let mut g = *c;
        for class in 0..classes {
            g.clock = class;
            game_write(ENTERED_RVA, 0u32);
            for (n, &p) in players.iter().enumerate() {
                game_write(CURRENT_RVA, n as u32);
                g.arg = get(p + 0xB0);
                g.palette = get(p + 0x90);
                if g.palette == class {
                    g.palette = class * 0x258 + 0x21C;
                    g.cars = table + g.palette;
                    // Converted: `rc2k_core::menu::bests::makes_score`.
                    if rc2k_core::menu::bests::makes_score(g.arg, get(g.cars + 0x38)) {
                        enter_best(None, 0xB0)?;
                        game_write(ENTERED_RVA, 1u32);
                    }
                }
            }
            g.measure_x = players.len() as u32;
            if game_read::<u32>(ENTERED_RVA) == 1 {
                game_write(CLASS_RVA, class);
                run_screen(&crate::menu::best_class_scores_loop::SCREEN, &g)?;
                if game_read::<u32>(ANSWER_RVA) == 1 {
                    break;
                }
            }
        }
    }
    Ok(())
}
