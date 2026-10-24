//! `menu::class_championship_pass` (0x4A4A1C): for each class from 1 (0 outside `dword_75AB40` 2 in event mode 0x13) up to `dword_7558F8`:
//! when a player (1 .. 3: the original does not look for player 4, section 16) is in the
//! class's entries of standings table B, the class numbered (`menu::number_standings_b`, its
//! count into `dword_75597C`) and its page shown (`menu::show_class_championship_standings`), stopping when the
//! leaving is answered yes (`g_yesNoAnswer` 1). `dword_755565` the class's flag.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers, `sub_4A9583`. The steps are their Rust versions; the page (inside `show_class_championship_standings`)
//! through `part` (1). Ungated (the pages wait for the player). Refuses a table without its
//! end entry and a class count the loop would run past (at or below the first, or past
//! 0x100), before anything; what the steps refuse. Self-test: a copy whose page wrapper
//! call goes to a stand-in, against the Rust version on the same, on the game's tables;
//! put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::Carried;
use crate::menu::standings::*;
use rc2k_core::menu::class_pages;

const FLAG_RVA: usize = rva(0x755565); // dword_755565
const COUNT_RVA: usize = rva(0x75597C); // dword_75597C
const CLASSES_RVA: usize = rva(0x7558F8); // dword_7558F8
const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer
const MODE_RVA: usize = rva(0x75AB40); // dword_75AB40
const EVENT_MODE_RVA: usize = rva(0x7556D0); // g_eventMode

/// The first class (see the header).
fn first_class() -> u32 {
    unsafe {
        class_pages::first_championship_class(game_read(MODE_RVA), || game_read(EVENT_MODE_RVA))
    }
}

/// The Rust version (see the header). `c` the carried values (registers the pages' screens are run with).
pub unsafe fn class_championship_pass(c: &Carried) -> Result<(), Refused> {
    unsafe {
        let first = first_class();
        // Converted: `rc2k_core::menu::class_pages` (`classes`, `has_player`).
        let classes = class_pages::classes(first, game_read(CLASSES_RVA))?;
        let table: u32 = game_read(TABLE_B_RVA);
        let all = entries(table, B_ENTRY)?;
        for class in classes {
            game_write(FLAG_RVA, 0u32);
            if !class_pages::has_player(
                all.len(),
                |n| get(all[n] + 0x10),
                |n| get(all[n] + 4),
                class,
            ) {
                continue;
            }
            game_write(FLAG_RVA, 1u32);
            let numbered = crate::menu::number_standings_b::number_standings_b(class)?;
            let mut g = Carried {
                palette: class,
                ..*c
            };
            numbered.carry_on(&mut g);
            game_write(COUNT_RVA, numbered.last_number);
            crate::menu::show_class_championship_standings::show_class_championship_standings(
                class, &g,
            )?;
            if game_read::<u32>(ANSWER_RVA) == 1 {
                break;
            }
        }
    }
    Ok(())
}
