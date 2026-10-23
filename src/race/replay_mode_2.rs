//! `race::replay_mode_2` (0x41CD9D): the replay viewer with eax 2 (a saved replay chosen
//! first), with no check of a replay there (unlike `replay_mode_1` / `_3`).
//!
//! In:  nothing
//! Out: eax 2, the rest as the viewer leaves them (kept)
//!
//! Caller: `sub_470529` (a menu item). Its call through `part` (0 the viewer; in the game
//! its Rust version). Ungated (the viewer runs for long and gates its own frames). Shadow
//! mode runs the original alone. Self-test: a copy whose call goes to a stand-in, against
//! the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::viewer::{Part, regs_entry};

/// The Rust version.
pub fn replay_mode_2(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = Regs { eax: 2, ..*entry };
    part(1, &mut r)?;
    Ok(r)
}

/// In the game: the viewer's Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::replay_viewer::{game_part, replay_viewer};
    *r = replay_viewer(r, &mut game_part)?;
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| replay_mode_2(r, &mut game_part))
}
