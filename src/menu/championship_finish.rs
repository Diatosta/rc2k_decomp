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
//! The finders' fatal errors are raised as the original's. Refuses the leader's ecx past 3 (the original reads past `g_players`) and what the
//! finders refuse, before anything.

use crate::errors::Refused;
use crate::menu::finish_calls::Game;
use rc2k_core::menu::finish;

pub fn championship_finish(menus: &crate::menu::owner::Menus) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::finish::championship_finish`.
    Game::run(menus, finish::championship_finish)
}
