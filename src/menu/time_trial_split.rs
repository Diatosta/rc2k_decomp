//! `menu::time_trial_split` (0x48AFD0): the time trial for two players sharing the screen
//! (event mode 4). `g_eventMode` 3, `menu::begin_event`, `clear_players_284`,
//! `take_player1_units`; `dword_755984` 2, `dword_755988` 4, `dword_755954` 3,
//! `dword_75555D` 0, `dword_7587B8`, `dword_7587BC`, `dword_7587A8`, `dword_7587AC` 0; the
//! two car choices (left, it ends at once). Then each stage: the current player 0,
//! `menu::ready_players`, the repairs of players 0 and 1, players 2 and 3's times (+0x88)
//! 0x36F268 and 0x36F650, the stage choice (`menu::time_trial_select_stage_loop`; a yes or
//! `dword_75555D` ends it), `dword_755938` 1, the service (`menu::time_trial_service`; its
//! yes is not looked at), the race (eax 0, ebx 3); unless both results (`dword_75B028`,
//! `dword_75B098`) are -1: the results page (`menu::time_trial_results_loop`; a yes ends
//! it), the stage's record in edi, both players' figures kept, `dword_755920` 0, the marks
//! cleared, each player's time entered when it qualifies (`rc2k_core::menu::rounds::qualify`); the best
//! times page when one did (a yes ends it); the review (`rc2k_core::menu::rounds::review`, eax 2; its yes
//! not looked at). The end: `rc2k_core::menu::rounds::end`.
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
    0xB, 0x10, 0x15, 0x7A, 0x7F, 0x91, 0xA6, 0xAB, 0xBD, 0xCC, 0xDB, 0xEA, 0x10F, 0x138, 0x147,
    0x160, 0x19E, 0x1AD, 0x1BC, 0x1D1, 0x1DF, 0x1EE, 0x1FC, 0x20A, 0x21D, 0x222, 0x266,
];

flow_hook!("menu::time_trial_split", 0x48AFD0, time_trial_split, test);

pub unsafe fn time_trial_split(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    players_there(4)?;
    // Converted: `rc2k_core::menu::rounds::time_trial_split`.
    rounds::time_trial_split(&mut Game::new(entry, call))?;
    Ok(*entry)
}

fn test(t: crate::framework::SelfTest) -> (usize, usize) {
    if stage_record().is_err() {
        return t.skip("no stage tables for the chosen stage");
    }
    let flow = Flow {
        address: 0x48AFD0,
        len: 0x26D,
        // pusha; mov g_eventMode, 3
        entry: &[
            0x60, 0xC7, 0x05, 0xD0, 0x56, 0x75, 0x00, 0x03, 0x00, 0x00, 0x00,
        ],
        calls: CALLS,
        fatal_jumps: &[],
        writes: &WRITES,
    };
    let race = |a: u32, b: u32| Answer::write(RACE_RESULT, a).and(RACE_RESULT_2, b);
    let mut rounds = crate::menu::time_trial_one::time_trial_scripts(1, TIME_TRIAL_RESULTS_LOOP);
    rounds.extend([
        (
            "both results -1, the review leaves".to_string(),
            vec![
                (RACE, vec![race(u32::MAX, u32::MAX)]),
                (REVIEW_485F5E, vec![Answer::write(LEAVE, 1)]),
            ],
        ),
        (
            "one result, the review's yes not looked at, then the stage choice's".to_string(),
            vec![
                (RACE, vec![race(u32::MAX, 4), race(2, 2)]),
                (REVIEW_485F5E, vec![Answer::yes()]),
            ],
        ),
    ]);
    test_rounds(t, &flow, 2, rounds, &|r, c| unsafe {
        time_trial_split(r, c)
    })
}
