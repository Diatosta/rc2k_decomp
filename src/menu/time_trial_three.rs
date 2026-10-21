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
//! Reached through the time trial's start menu. The first five bytes hold no jump target
//! or call. Ungated (a whole event). Refuses a missing player before anything, and what
//! the callees and the stage's record (`Hand::Stage`) refuse when they are reached. Shadow mode runs the
//! original alone. Self-test: `rounds::test_rounds`.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::event_flow::*;
use crate::menu::flow_game::Game;
use crate::menu::rounds::*;
use rc2k_core::menu::rounds;

const CALLS: &[usize] = &[
    0xB, 0x10, 0x15, 0xAC, 0xB1, 0xC3, 0xD8, 0xDD, 0xF5, 0xFA, 0x10F, 0x114, 0x12C, 0x131, 0x140,
    0x14F, 0x15E, 0x16D, 0x17C, 0x1A1, 0x1CA, 0x1E3, 0x1F1, 0x1F6, 0x21A, 0x228, 0x22D, 0x254,
    0x262, 0x267, 0x282, 0x2C0, 0x2CF, 0x2DE, 0x2ED, 0x302, 0x310, 0x31F, 0x32D, 0x33C, 0x34A,
    0x358, 0x36B, 0x370, 0x3BD,
];

flow_hook!("menu::time_trial_three", 0x48B54B, time_trial_three, test);

pub unsafe fn time_trial_three(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    players_there(4)?;
    // Converted: `rc2k_core::menu::rounds::time_trial_three`.
    rounds::time_trial_three(&mut Game::new(entry, call))?;
    Ok(*entry)
}

fn test(t: crate::framework::SelfTest) -> (usize, usize) {
    if stage_record().is_err() {
        return t.skip("no stage tables for the chosen stage");
    }
    let flow = Flow {
        address: 0x48B54B,
        len: 0x3C4,
        // pusha; mov g_eventMode, 3
        entry: &[
            0x60, 0xC7, 0x05, 0xD0, 0x56, 0x75, 0x00, 0x03, 0x00, 0x00, 0x00,
        ],
        calls: CALLS,
        fatal_jumps: &[],
        writes: &WRITES,
    };
    let rounds = crate::menu::time_trial_one::time_trial_scripts(3, TIME_TRIAL_RESULTS_LOOP);
    test_rounds(t, &flow, 3, rounds, &|r, c| unsafe {
        time_trial_three(r, c)
    })
}
