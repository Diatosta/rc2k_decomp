//! `menu::time_trial_three` (0x48B54B): the time trial for three players in turn (event
//! mode 0x16). `g_eventMode` 3, `menu::begin_event`, `clear_players_284`,
//! `take_player1_units`; `dword_758714` 0, `dword_755984` 3, `dword_755988` 0x16,
//! `dword_755954` 3, `dword_75555D` 0, the eight dwords `dword_7587A8` .. `dword_7587C4`
//! 0; the three car choices (players 1 and 2's list made again before their car is
//! taken; left, it ends at once). Then each stage: the current player 0,
//! `menu::ready_players`, the four repairs, players 2 and 3's times (+0x88) 0x36F268 and
//! 0x36F650, the stage choice (a yes or `dword_75555D` ends it), `dword_755938` 1, the
//! service (its yes not looked at), the three races (`rc2k_core::menu::rounds::race_series`, ebx 3; a race
//! left, -2, goes to the review); unless all results are -1: the results page (a yes ends
//! it), the stage's record in edi, the three figures kept, `dword_755920` 0, the marks
//! cleared, each time entered when it qualifies, the best times page when one did (a yes
//! ends it); the review (eax 3, a yes asked). The end: `rc2k_core::menu::rounds::end`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through the time trial's start menu. Refuses a missing player before anything, and what
//! the callees and the stage's record (`Hand::Stage`) refuse when they are reached.

use crate::errors::Refused;
use crate::menu::flow_game::Game;
use rc2k_core::menu::rounds;

pub fn time_trial_three() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::rounds::time_trial_three`.
    Game::new().run(rounds::time_trial_three)
}
