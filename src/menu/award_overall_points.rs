//! `menu::award_overall_points` (0x4A4933): the stage's overall championship points. The
//! points start at 0x14 (`seconds`); each entry of standings table B whose +0x10 is not 0,
//! in the table's order, gets them into its +0x38 + 4 * `section` and they go down by one
//! (not below 0); a computer's driver (+4 0, `menu::find_cpu_team` by its +0) also into
//! its +0x5C + 4 * `section`, a player (+4 the player + 1) into its +0xB4 + 4 * `section`.
//! Then each entry's total of its six (+0x38 .., the negative ones left out) into its
//! +0x50 and its driver's +0x74 or player's +0xCC.
//!
//! In:  nothing
//! Out: edi the end entry, ecx 6, edx the last total, eax the last entry's driver number
//!      or player index, esi its driver or player (no `pusha`)
//!
//! Caller: `sub_4A0E60`. The search is its Rust version. Refuses no table, one without its
//! end entry, a section past 5, a player past 4 or missing, before anything; a driver not
//! found is the search's fatal error, before anything. Self-test: the original and the
//! Rust version on the game's tables, groups and sections by lot; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const POINTS_RVA: usize = rva(0x7559AC); // seconds
/// An entry's driver (with its number) or player (with its index).
unsafe fn owner(entry: &Regs, e: u32) -> Result<(u32, u32, bool), Refused> {
    match get(e + 4) {
        0 => {
            let g = unsafe {
                crate::menu::find_cpu_team::find_cpu_team(&Regs {
                    eax: get(e),
                    edi: e,
                    ..*entry
                })
            }?;
            Ok((g.esi, get(e), true))
        }
        n if n <= 4 && player(n - 1) != 0 => Ok((player(n - 1), n - 1, false)),
        _ => Err(BadInput("a player past 4 or missing").into()),
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn award_overall_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let section: u32 = game_read(SECTION_RVA);
        let table: u32 = game_read(TABLE_B_RVA);
        if table == 0 || section > 5 {
            return Err(BadInput("no table, or a section past 5").into());
        }
        let all = entries(table, B_ENTRY)?;
        let mut owners = Vec::with_capacity(all.len());
        for &e in &all {
            owners.push(owner(entry, e)?);
        }
        // Converted: `rc2k_core::menu::points` (`Points`, `total`).
        use rc2k_core::menu::points::{Points, total};
        let at = 4 * section;
        let mut points = Points::default();
        game_write(POINTS_RVA, points.0);
        for (&e, &(o, _, cpu)) in all.iter().zip(&owners) {
            if get(e + 0x10) == 0 {
                continue;
            }
            let given = points.take();
            game_write(POINTS_RVA, points.0);
            put(e + 0x38 + at, given);
            put(o + if cpu { 0x5C } else { 0xB4 } + at, given);
        }
        for (&e, &(o, number, cpu)) in all.iter().zip(&owners) {
            let total = total(std::array::from_fn(|k| get(e + 0x38 + 4 * k as u32)));
            put(e + 0x50, total);
            put(o + if cpu { 0x74 } else { 0xCC }, total);
            g.eax = number;
            g.esi = o;
            g.edx = total;
        }
        g.ecx = 6;
        g.edi = table + B_ENTRY * all.len() as u32;
    }
    Ok(g)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { award_overall_points(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
