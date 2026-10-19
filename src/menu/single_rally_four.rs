//! `menu::single_rally_four` (0x482B7A): the single rally for four players in turn (event
//! mode 8). `menu::begin_event`, `clear_players_284`, `take_player1_units`,
//! `clear_event_state`; `dword_755984` 4, `dword_755988` 8 (neither `dword_75555D` nor
//! `dword_758714` set to 0, unlike the three players'); the four car choices (one left
//! ends it at once). Then each round as `menu::single_rally_three`'s
//! (`single_rally_three::rally_rounds`) but without the current player set to 0 before
//! `menu::ready_players` and without the fourth player's time: four races, eax 4 for the
//! review. The end: `rounds::end`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through the start menu's table. A call at +1: a private copy (0x321 bytes, its
//! 39 calls relocated) and a jump written by hand. Ungated (a whole event). Refuses a
//! missing player before anything, and what the callees refuse when they are reached.
//! Shadow mode runs the original alone. Self-test: `rounds::test_rounds`.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::event_flow::*;
use crate::menu::rounds::*;

const CALLS: &[usize] = &[
    0x1, 0x6, 0xB, 0x10, 0x58, 0x6A, 0x9E, 0xB0, 0xE4, 0xF6, 0x12A, 0x13C, 0x141, 0x150, 0x15F,
    0x16E, 0x17D, 0x182, 0x1AB, 0x1BA, 0x1C8, 0x1CD, 0x1E7, 0x1F6, 0x204, 0x209, 0x226, 0x235,
    0x243, 0x248, 0x262, 0x271, 0x27F, 0x284, 0x298, 0x2AF, 0x2C2, 0x2C7, 0x31A,
];

flow_hook!(
    "menu::single_rally_four",
    0x482B7A,
    single_rally_four,
    test,
    copy(0x321, CALLS)
);

pub unsafe fn single_rally_four(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    players_there(4)?;
    let mut g = *entry;
    call(BEGIN_EVENT, &mut g)?;
    call(CLEAR_PLAYERS_284, &mut g)?;
    call(TAKE_PLAYER1_UNITS, &mut g)?;
    call(CLEAR_EVENT_STATE, &mut g)?;
    set(PLAYER_COUNT, 4);
    set(EVENT_MODE, 8);
    if !choose_cars(&mut g, call, 4, false, &[])? {
        return Ok(*entry);
    }
    unsafe { crate::menu::single_rally_three::rally_rounds(&mut g, call, 4)? };
    end(&mut g, call)?;
    Ok(*entry)
}

fn test(t: crate::framework::SelfTest) -> (usize, usize) {
    let flow = Flow {
        address: 0x482B7A,
        len: 0x321,
        // pusha; call menu::begin_event
        entry: &[0x60, 0xE8, 0x2A, 0x00, 0xFF, 0xFF],
        calls: CALLS,
        fatal_jumps: &[],
        writes: &WRITES,
    };
    let mut rounds = crate::menu::single_rally_two::round_scripts(4, RESULTS_482E9B);
    rounds.push((
        "two rounds, the second's overall results' yes".into(),
        vec![
            (
                RACE,
                (0..8).map(|k| Answer::write(RACE_RESULT, k)).collect(),
            ),
            (SCREEN_483163, vec![Answer::yes()]),
        ],
    ));
    test_rounds(t, &flow, 4, rounds, &|r, c| unsafe {
        single_rally_four(r, c)
    })
}
