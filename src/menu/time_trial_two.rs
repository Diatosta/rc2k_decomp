//! `menu::time_trial_two` (0x48B23D): the time trial for two players in turn (event mode
//! 0x15). As `menu::time_trial_split`'s start (`dword_755988` 0x15). Then each stage: the
//! current player 0, `menu::ready_players`, the four repairs, players 2 and 3's times
//! (+0x88, through `dword_75598C` 2 and 3) 0x36F268 and 0x36F650, the stage choice (a yes
//! or `dword_75555D` ends it), `dword_755938` 1, the service (its yes not looked at), the
//! two races (`rc2k_core::menu::rounds::race_series`, ebx 3; a race left, -2, goes to the review); when
//! the results are not all -1: the results page (`menu::time_trial_results_loop`; a yes
//! ends it), the stage's record in edi, both figures kept, `dword_755920` 0, the marks
//! cleared, player 0's time entered when it qualifies; then (also when all were -1, the
//! original's jump goes there) player 1's; the best times page when one qualified (a yes
//! ends it); the review (eax 2, a yes asked). The end: `rc2k_core::menu::rounds::end`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through the time trial's start menu. Refuses a missing player before anything, and
//! what the callees and the stage's record (`Hand::Stage`) refuse when they are reached.
//! Hook removed 2026-10-22 (typed tables): the start menus run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::event_flow::*;
use crate::menu::flow_game::Game;
use rc2k_core::menu::rounds;

flow_hook!("menu::time_trial_two", 0x48B23D, time_trial_two, test);

pub unsafe fn time_trial_two(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    players_there(4)?;
    // Converted: `rc2k_core::menu::rounds::time_trial_two`.
    rounds::time_trial_two(&mut Game::new(entry, call))?;
    Ok(*entry)
}
