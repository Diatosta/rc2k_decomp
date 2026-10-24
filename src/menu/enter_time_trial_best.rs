//! `menu::enter_time_trial_best` (0x48C8B6): the current player's time put in the tenth
//! of the chosen stage's best times and moved up to its place. `dword_7559EC` the stage's
//! +0x3C; the tenth entry's +0x14 the player's time (+0x88), +0x1C its +0x90, +8 the
//! nationality (+4), +0xC and +0x20 the name pointer (+0x18), +0x10 the car (+8), +4 1,
//! +0x40 1 (the new entry's mark), +0x50 `g_cheats`, +0x54 / +0x58 the sealed time
//! (`menu::sealed_player_time` with eax the stage's +0x3C), the 0x40 bytes at the player's
//! +0x20C at +0x5C, the name's first 31 bytes at +0x20; then `menu::rank_time_trial_best`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: the time trial's events, when the player qualifies. The two callees are their
//! Rust versions, `sealed_player_time` with the registers the original has there (eax the
//! stage's +0x3C, ebx `dword_7558D0`, esi the player, edi the entry). Refuses (before
//! anything) a current player past the four or missing, a stage the tables do not give,
//! or a name pointer of 0. Self-test: the game's tables for each stage choice there, the
//! player's time by case; the seal's state, `dword_7559EC`, `hhmm` and `g_keptPlayerName`
//! compared too; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::menu::event_flow::*;

const STAGE_SLOT: usize = 0x7559EC; // dword_7559EC
// hhmm

/// The Rust version (see the header).
pub unsafe fn enter_time_trial_best() -> Result<(), Refused> {
    use rc2k_core::menu::bests;
    let record = stage_record()?;
    stage_times()?; // a stage the tables do not give: refused
    let p = current_player()?;
    let name = dword(p + 0x18);
    if name == 0 {
        return Err(BadInput("a player's name pointer of 0").into());
    }
    let slot = dword(record + 0x3C);
    set(STAGE_SLOT, slot);
    let cheats = get(CHEATS);
    let sealed = unsafe { crate::menu::sealed_player_time::sealed_player_time(slot)? };
    // Converted: `rc2k_core::menu::bests::enter_tenth`, then `rank_time_trial_best`.
    use crate::menu::standings::bytes;
    let best = bests::NewBest {
        key: dword(p + 0x88),
        player: 1,
        nationality: dword(p + 4),
        name_at: name,
        car: dword(p + 8),
        group: dword(p + 0x90),
        name: bytes(name),
        entered: None,
        seal: Some(bests::Seal { cheats, sealed }),
        line: Some(bytes(p + 0x20C)),
    };
    bests::enter_tenth(&mut crate::menu::rank_time_trial_best::stage_list()?, &best);
    crate::menu::rank_time_trial_best::rank_time_trial_best()
}
