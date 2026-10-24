//! `menu::championship_finish` (0x4A14CB): the championship finish routine. `dword_75B108`
//! and `dword_75ACE4` 0; the leader looked for in standings table B (event mode 0x13:
//! `menu::first_player_b` in group 0; otherwise `menu::first_player_unplaced`): when its
//! edx is 1, `dword_75ACCC` and `dword_75B108` 1 and the player `g_players`[ecx] (ecx as
//! the finder leaves it: section 16); otherwise `dword_75B108` 0 and the first player done
//! with the section (`menu::find_section_player`; none: nothing but a key taken). Then
//! the race's set-up from it (`finish_calls::set_up`), the sounds' and screens' set-up,
//! `drive::finish_race_0`, `dword_75B108` 0, the front end's set-up again (with the
//! race's sounds once more); a key taken at the end (eax 0).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A0E60`. The finders are their Rust versions (their fatal errors raised
//! as the original's); the calls through `part`, in the game through their addresses.
//! Ungated. Refuses the leader's ecx past 3 (the original reads past `g_players`) and what
//! the finders refuse, before anything. Shadow mode runs the original alone. Self-test: a
//! copy whose calls go to stand-ins, against the Rust version, in both event modes.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::menu::finish_calls::Game;
use rc2k_core::menu::finish;

/// The Rust version (see the header).
pub fn championship_finish() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::finish::championship_finish`.
    finish::championship_finish(&mut Game::default())
}
