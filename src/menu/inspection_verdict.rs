//! `menu::inspection_verdict` (0x4A1952): the current player's (`dword_75598C`) inspection
//! verdict. The penalties (`dword_75AC48` ..) of its inspected parts at or past their
//! limits whose flag in `dword_759B6E` (index + 2) is 0 added into `dword_7559EC`; 100
//! less that into `dword_75AC80`; the player's +0xD0 1 when that is at least 100 (no
//! penalty), else 2.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A8556`. Refuses a current player past 3 or missing, before anything.
//! Self-test: the original and the Rust version on the game's player, parts around their
//! limits and flags by lot; put back.
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

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const FLAGS_RVA: usize = rva(0x759B6E); // dword_759B6E
const SUM_RVA: usize = rva(0x7559EC); // dword_7559EC
const SCORE_RVA: usize = rva(0x75AC80); // dword_75AC80
/// The Rust version (see the header): the registers it leaves.
pub unsafe fn inspection_verdict(entry: &Regs) -> Result<Regs, Refused> {
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    // Converted: `rc2k_core::menu::inspection::verdict`.
    let p = player(n);
    unsafe {
        let v = rc2k_core::menu::inspection::verdict(
            |k| past_limit(p, k as u32),
            |k| game_read::<u32>(FLAGS_RVA + 4 * (k + 2)) != 0,
            |k| game_read(PENALTIES_RVA + 4 * k),
        );
        game_write(SUM_RVA, v.penalties);
        game_write(SCORE_RVA, v.score);
        put(p + 0xD0, v.kind);
    }
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { inspection_verdict(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
