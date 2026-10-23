//! `race::game_music` (0x470F8A): the race's tune: 6 with `dword_75B108` 1, else 5.
//!
//! In:  nothing
//! Out: every register preserved (the tunes preserve them)
//!
//! See `race/music.rs`. One caller, `race::load_stage`. Shadow mode runs the original
//! alone (a command must not be sent twice). Self-test: a redirected copy whose tune
//! calls note themselves and their registers, `dword_75B108` 0, 1, 2.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::music::{game_music, game_tune};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    game_music(r, &mut game_tune)?;
    Ok(())
}
