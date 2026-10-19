//! `menu::time_trial_four` (0x48B90F): the time trial for four players in turn (event mode
//! 0x17). `g_eventMode` 3, `menu::begin_event`, `clear_players_284`,
//! `take_player1_units`; `dword_755984` 4, `dword_755988` 0x17, `dword_755954` 3,
//! `dword_75555D` 0 (no other variables cleared); the four car choices (player 0's list
//! made again before its car is taken; left, it ends at once). Then each stage as
//! `menu::time_trial_three`'s (`time_trial_three::time_trial_rounds`) with four races and
//! these differences: no current player 0 before `menu::ready_players`, the service's yes
//! ends it, no stage record put in edi, and before the review `menu::show_in_game_options`
//! where the others hide them. The end: `rounds::end`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through the time trial's start menu. The first five bytes hold no jump target
//! or call. Ungated (a whole event). Refuses a missing player before anything, and what
//! the callees refuse when they are reached. Shadow mode runs the original alone.
//! Self-test: `rounds::test_rounds`.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::event_flow::*;
use crate::menu::rounds::*;
use crate::menu::time_trial_split::time_trial_start;

const CALLS: &[usize] = &[
    0xB, 0x10, 0x15, 0x52, 0x57, 0x6F, 0x74, 0x89, 0x8E, 0xA0, 0xB5, 0xBA, 0xCC, 0xE1, 0xE6, 0xF8,
    0xFD, 0x10C, 0x11B, 0x12A, 0x139, 0x15E, 0x187, 0x1AD, 0x1BB, 0x1C0, 0x1E4, 0x1F2, 0x1F7,
    0x21E, 0x22C, 0x231, 0x258, 0x266, 0x26B, 0x286, 0x2A2, 0x2B1, 0x2C0, 0x2CF, 0x2DE, 0x2F3,
    0x301, 0x310, 0x31E, 0x32D, 0x33B, 0x34A, 0x358, 0x366, 0x374, 0x379, 0x3C6,
];

flow_hook!("menu::time_trial_four", 0x48B90F, time_trial_four, test);

pub unsafe fn time_trial_four(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    players_there(4)?;
    let mut g = *entry;
    time_trial_start(&mut g, call)?;
    set(PLAYER_COUNT, 4);
    set(EVENT_MODE, 0x17);
    set(EVENT_TYPE, 3);
    set(LEAVE, 0);
    if !choose_cars(&mut g, call, 4, true, &[0])? {
        return Ok(*entry);
    }
    unsafe { crate::menu::time_trial_three::time_trial_rounds(&mut g, call, 4, true)? };
    end(&mut g, call)?;
    Ok(*entry)
}

fn test(t: crate::framework::SelfTest) -> (usize, usize) {
    let flow = Flow {
        address: 0x48B90F,
        len: 0x3CD,
        // pusha; mov g_eventMode, 3
        entry: &[
            0x60, 0xC7, 0x05, 0xD0, 0x56, 0x75, 0x00, 0x03, 0x00, 0x00, 0x00,
        ],
        calls: CALLS,
        fatal_jumps: &[],
        writes: &WRITES,
    };
    let rounds = crate::menu::time_trial_one::time_trial_scripts(4, TIME_TRIAL_RESULTS_LOOP);
    test_rounds(t, &flow, 4, rounds, &|r, c| unsafe {
        time_trial_four(r, c)
    })
}
