//! `menu::single_rally_two` (0x4826A2): the single rally for two players in turn (event
//! mode 8). `menu::begin_event`, `clear_players_284`, `take_player1_units`;
//! `dword_755984` 2, `dword_755988` 8, `dword_75555D` 0; `clear_event_state`; the two car
//! choices (one left ends it at once). Then each stage: the current player 0,
//! `menu::ready_players`, the four repairs, the stage choice (`menu::select_stage_loop`; a
//! yes or `dword_75555D` ends it); the two races (`rounds::race_series`, ebx 0, each car
//! repaired first; a race left, -2, goes to the review); unless all results are -1 the
//! results page (`menu::results_48584b_loop`; a yes ends it); the review (`rounds::review`,
//! eax 2, a yes asked). The end: `rounds::end`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through the start menu's table. A call at +1: a private copy (0x20E bytes, its
//! 26 calls relocated) and a jump written by hand. Ungated (a whole event). Refuses a
//! missing player before anything, and what the callees refuse when they are reached.
//! Shadow mode runs the original alone. Self-test: `rounds::test_rounds`.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::event_flow::*;
use crate::menu::rounds::*;

const CALLS: &[usize] = &[
    0x1, 0x6, 0xB, 0x2E, 0x62, 0x74, 0xA8, 0xBA, 0xC9, 0xD8, 0xE7, 0xF6, 0x105, 0x10A, 0x133,
    0x142, 0x150, 0x155, 0x16C, 0x17B, 0x189, 0x18E, 0x1A2, 0x1B5, 0x1BA, 0x207,
];

flow_hook!(
    "menu::single_rally_two",
    0x4826A2,
    single_rally_two,
    test,
    copy(0x20E, CALLS)
);

pub unsafe fn single_rally_two(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    players_there(4)?;
    let mut g = *entry;
    call(BEGIN_EVENT, &mut g)?;
    call(CLEAR_PLAYERS_284, &mut g)?;
    call(TAKE_PLAYER1_UNITS, &mut g)?;
    set(PLAYER_COUNT, 2);
    set(EVENT_MODE, 8);
    set(LEAVE, 0);
    call(CLEAR_EVENT_STATE, &mut g)?;
    if !choose_cars(&mut g, call, 2, false, &[])? {
        return Ok(*entry);
    }
    loop {
        set(CURRENT, 0);
        call(READY_PLAYERS, &mut g)?;
        repair_all(&mut g, call)?;
        call(SELECT_STAGE_LOOP, &mut g)?;
        if yes() || get(LEAVE) == 1 {
            break;
        }
        if let Some(all) = race_series(&mut g, call, 2, 0, true)?
            && all != u32::MAX
        {
            call(RESULTS_48584B, &mut g)?;
            if yes() {
                break;
            }
        }
        if review(&mut g, call, Some(2), true)? {
            break;
        }
    }
    end(&mut g, call)?;
    Ok(*entry)
}

/// The multi-player rounds' scripts, for `n` players (`results` the results page).
pub fn round_scripts(n: u32, results: usize) -> Vec<(String, Script)> {
    let race = |v: u32| Answer::write(RACE_RESULT, v);
    let mut out: Vec<(String, Script)> = vec![
        (
            "the stage choice's yes".into(),
            vec![(SELECT_STAGE_LOOP, vec![Answer::yes()])],
        ),
        (
            "the stage choice left".into(),
            vec![(SELECT_STAGE_LOOP, vec![Answer::write(LEAVE, 1)])],
        ),
        (
            "every race, the results, the review ends".into(),
            vec![
                (RACE, (0..n).map(|k| race(0x10 + k)).collect()),
                (REVIEW_485F5E, vec![Answer::write(ENDED_569, 1)]),
            ],
        ),
        (
            "no results, the review's yes".into(),
            vec![
                (RACE, (0..n).map(|_| race(u32::MAX)).collect()),
                (REVIEW_485F5E, vec![Answer::yes()]),
            ],
        ),
        (
            "the results page's yes".into(),
            vec![
                (RACE, (0..n).map(race).collect()),
                (results, vec![Answer::yes()]),
            ],
        ),
        (
            "two rounds, then the stage choice's yes".into(),
            vec![
                (RACE, (0..2 * n).map(|k| race(k * 3)).collect()),
                (
                    SELECT_STAGE_LOOP,
                    vec![Answer::none(), Answer::none(), Answer::yes()],
                ),
            ],
        ),
    ];
    for k in 0..n {
        let mut races: Vec<Answer> = (0..k).map(|j| race(j + 1)).collect();
        races.push(race(0xFFFF_FFFE));
        out.push((
            format!("race {k} left (-2), then the review leaves"),
            vec![
                (RACE, races),
                (REVIEW_485F5E, vec![Answer::write(LEAVE, 1)]),
            ],
        ));
    }
    out
}

fn test(t: crate::framework::SelfTest) -> (usize, usize) {
    let flow = Flow {
        address: 0x4826A2,
        len: 0x20E,
        // pusha; call menu::begin_event
        entry: &[0x60, 0xE8, 0x02, 0x05, 0xFF, 0xFF],
        calls: CALLS,
        fatal_jumps: &[],
        writes: &WRITES,
    };
    test_rounds(
        t,
        &flow,
        2,
        round_scripts(2, RESULTS_48584B),
        &|r, c| unsafe { single_rally_two(r, c) },
    )
}
