//! `race::save_last_replay` (0x41ECFF): the last race's replay saved as replay eax: opened
//! (`race::open_viewed_replay`, 222), saved (`race::save_viewed_replay`, ebx the slot) and
//! closed (`race::close_viewed_replay`).
//!
//! In:  eax the slot
//! Out: carry set when the opening or the saving failed (the saving's flags kept round
//!      the closing); every register kept (`pusha` / `popa`)
//!
//! Caller: `race::replay_viewer` (mode 3, the slot from `menu::save_replay_loop`; no flags
//! read). Its calls through `part`: 1 `open_viewed_replay` (ebx 222), 2
//! `save_viewed_replay` (ebx the slot), 3 `close_viewed_replay`; in the game their Rust
//! versions. Shadow mode runs the original alone (it writes a file). Self-test: a copy
//! whose calls go to stand-ins, against the Rust version: the opening failing, the saving
//! failing or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::viewer::{LAST_RACE, regs_entry};

/// The Rust version: the registers and flags it leaves.
pub fn save_last_replay(entry: &Regs) -> Result<Regs, Refused> {
    let mut out = *entry;
    let mut r = Regs {
        ebx: LAST_RACE,
        ..*entry
    };
    crate::race::save_last_replay::game_part(1, &mut r)?;
    if r.carry() {
        out.eflags = r.eflags;
        return Ok(out);
    }
    r.ebx = r.eax;
    crate::race::save_last_replay::game_part(2, &mut r)?;
    let flags = r.eflags;
    crate::race::save_last_replay::game_part(3, &mut r)?;
    out.eflags = flags;
    Ok(out)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::{close_viewed_replay as c, open_viewed_replay as o, save_viewed_replay as s};
    *r = match place {
        1 => o::open_viewed_replay(r, &mut o::game_part)?,
        2 => s::save_viewed_replay(r, &mut s::game_part)?,
        _ => c::close_viewed_replay(r)?,
    };
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| save_last_replay(r))
}
