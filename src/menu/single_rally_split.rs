//! `menu::single_rally_split` (0x4824C1): the single rally for two players sharing the
//! screen (event mode 5). `menu::begin_event`, `clear_players_284`, `take_player1_units`;
//! `dword_755984` 2, `dword_755988` 5, `dword_75555D` 0; `clear_event_state`; the two car
//! choices (`rc2k_core::menu::rounds::choose_cars`; one left ends it at once). Then each stage: the current
//! player 0, `menu::ready_players`, the four repairs, the stage choice
//! (`menu::select_stage_loop`; a yes or `dword_75555D` ends it); player 0 taking part
//! (+0xD0 1), the repairs with ecx 0 and 1, the race (`race::single_rally_race`, eax 0,
//! ebx 0); unless both results (`dword_75B028`, `dword_75B098`) are -1 the results page
//! (`menu::results_48584b_loop`; a yes ends it); the review (`rc2k_core::menu::rounds::review`, eax 2; no
//! yes asked). The end: `rc2k_core::menu::rounds::end`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through the start menu's table. A call at +1: a private copy (0x1E1 bytes, its
//! 21 calls relocated) and a jump written by hand. Ungated (a whole event). Refuses a
//! missing player before anything, and what the callees refuse when they are reached.
//! Shadow mode runs the original alone. Self-test: `rounds::test_rounds`.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::event_flow::*;
use crate::menu::flow_game::Game;
use crate::menu::rounds::*;
use rc2k_core::menu::rounds;

const CALLS: &[usize] = &[
    0x1, 0x6, 0xB, 0x2E, 0x62, 0x74, 0xA8, 0xBA, 0xC9, 0xD8, 0xE7, 0xF6, 0x105, 0x10A, 0x150,
    0x15A, 0x169, 0x17E, 0x191, 0x196, 0x1DA,
];

flow_hook!(
    "menu::single_rally_split",
    0x4824C1,
    single_rally_split,
    test,
    copy(0x1E1, CALLS)
);

pub unsafe fn single_rally_split(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    players_there(4)?;
    // Converted: `rc2k_core::menu::rounds::single_rally_split`.
    rounds::single_rally_split(&mut Game::new(entry, call))?;
    Ok(*entry)
}

fn test(t: crate::framework::SelfTest) -> (usize, usize) {
    let flow = Flow {
        address: 0x4824C1,
        len: 0x1E1,
        // pusha; call menu::begin_event
        entry: &[0x60, 0xE8, 0xE3, 0x06, 0xFF, 0xFF],
        calls: CALLS,
        fatal_jumps: &[],
        writes: &WRITES,
    };
    let race = |a: u32, b: u32| Answer::write(RACE_RESULT, a).and(RACE_RESULT_2, b);
    let rounds: Vec<(String, Script)> = vec![
        (
            "the stage choice's yes".into(),
            vec![(SELECT_STAGE_LOOP, vec![Answer::yes()])],
        ),
        (
            "a race, its results, the review ends".into(),
            vec![
                (RACE, vec![race(5, 6)]),
                (REVIEW_485F5E, vec![Answer::write(ENDED_569, 1)]),
            ],
        ),
        (
            "no results, the review leaves".into(),
            vec![
                (RACE, vec![race(u32::MAX, u32::MAX)]),
                (REVIEW_485F5E, vec![Answer::write(LEAVE, 1)]),
            ],
        ),
        (
            "the results page's yes".into(),
            vec![
                (RACE, vec![race(u32::MAX, 3)]),
                (RESULTS_48584B, vec![Answer::yes()]),
            ],
        ),
        (
            "two stages, a yes on the review not asked".into(),
            vec![
                (RACE, vec![race(1, 2), race(3, 4)]),
                (REVIEW_485F5E, vec![Answer::yes()]),
                (
                    SELECT_STAGE_LOOP,
                    vec![Answer::none(), Answer::none(), Answer::write(LEAVE, 1)],
                ),
            ],
        ),
    ];
    test_rounds(t, &flow, 2, rounds, &|r, c| unsafe {
        single_rally_split(r, c)
    })
}
