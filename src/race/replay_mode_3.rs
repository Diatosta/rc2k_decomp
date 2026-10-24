//! `race::replay_mode_3` (0x41CD89): with a replay there (`dword_5BEC84` not 0), the replay
//! viewer `race::replay_viewer` with eax 3 (see `race/replay_mode.rs`).
//!
//! In:  nothing
//! Out: eax 3 and the registers as the viewer leaves them, with a replay; else all kept
//!
//! Callers: the review menus' replay item (`menu::replay_item_b`). The viewer through `part`
//! (in the game its Rust version). Ungated (the viewer runs for long and gates its own
//! frames). Shadow mode runs the original alone (the viewer plays the replay). Self-test: `replay_mode::test_mode`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::menu::screen_loop::Carried;
use crate::race::replay_mode::replay_mode;

/// The Rust version (`c` the carried values the item gets).
pub unsafe fn replay_mode_3(c: &Carried) -> Result<(), Refused> {
    unsafe { replay_mode(c, 3) }
}
