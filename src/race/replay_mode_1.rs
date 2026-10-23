//! `race::replay_mode_1` (0x41CD75): with a replay there (`dword_5BEC84` not 0), the replay
//! viewer `race::replay_viewer` with eax 1 (see `race/replay_mode.rs`).
//!
//! In:  nothing
//! Out: eax 1 and the registers as the viewer leaves them, with a replay; else all kept
//!
//! Callers: the review menus' replay items (`menu::replay_item_a`) and 0x4430EF. The viewer through `part`
//! (in the game its Rust version). Ungated (the viewer runs for long and gates its own
//! frames). Shadow mode runs the original alone (the viewer plays the replay). Self-test: `replay_mode::test_mode`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::replay_mode::{game_part, replay_mode};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { replay_mode(r, 1, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
