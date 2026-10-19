//! `menu::single_rally_three` (0x4828B0): the single rally for three players in turn
//! (event mode 8). `menu::begin_event`, `clear_players_284`, `take_player1_units`,
//! `clear_event_state`; `dword_758714` (the rounds) 0, `dword_755984` 3, `dword_755988`
//! 8, `dword_75555D` 0; the three car choices (one left ends it at once). Then each
//! round: the current player 0, `menu::ready_players`, the four repairs, the fourth
//! player's time (+0x88) 0x36F650, the stage choice (`menu::select_stage_loop`; a yes or
//! `dword_75555D` ends it); the three races (`rounds::race_series`, ebx 0, each car
//! repaired first; a race left, -2, goes to the review); unless all results are -1 the
//! results page (`menu::results_482e9b_loop`; a yes ends it) and from the second round
//! on the overall results (`menu::screen_483163_loop`; a yes ends it); the review
//! (`rounds::review`, eax 3, a yes asked); `dword_758714` + 1. The end: `rounds::end`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through the start menu's table. A call at +1: a private copy (0x2CA bytes, its
//! 33 calls relocated) and a jump written by hand. Ungated (a whole event). Refuses a
//! missing player before anything, and what the callees refuse when they are reached.
//! Shadow mode runs the original alone. Self-test: `rounds::test_rounds`.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::event_flow::*;
use crate::menu::rounds::*;

const CALLS: &[usize] = &[
    0x1, 0x6, 0xB, 0x10, 0x6C, 0x7E, 0xB2, 0xC4, 0xF8, 0x10A, 0x119, 0x128, 0x137, 0x146, 0x155,
    0x16A, 0x193, 0x1A2, 0x1B0, 0x1B5, 0x1CF, 0x1DE, 0x1EC, 0x1F1, 0x20B, 0x21A, 0x228, 0x22D,
    0x241, 0x258, 0x26B, 0x270, 0x2C3,
];

flow_hook!(
    "menu::single_rally_three",
    0x4828B0,
    single_rally_three,
    test,
    copy(0x2CA, CALLS)
);

/// The three and four players' rounds after the car choices (`menu::single_rally_four`
/// starts them without the current player 0 and the fourth's time).
pub unsafe fn rally_rounds(g: &mut Regs, call: Call, n: u32) -> Result<(), Refused> {
    loop {
        if n == 3 {
            set(CURRENT, 0);
        }
        call(READY_PLAYERS, g)?;
        repair_all(g, call)?;
        if n == 3 {
            g.edi = player(3);
            put(g.edi + 0x88, TIME_3);
        }
        call(SELECT_STAGE_LOOP, g)?;
        if yes() || get(LEAVE) == 1 {
            return Ok(());
        }
        if let Some(all) = race_series(g, call, n, 0, true)?
            && all != u32::MAX
        {
            call(RESULTS_482E9B, g)?;
            if yes() {
                return Ok(());
            }
            if get(ROUND) != 0 {
                call(SCREEN_483163, g)?;
                if yes() {
                    return Ok(());
                }
            }
        }
        if review(g, call, Some(n), true)? {
            return Ok(());
        }
        inc(ROUND);
    }
}

pub unsafe fn single_rally_three(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    players_there(4)?;
    let mut g = *entry;
    call(BEGIN_EVENT, &mut g)?;
    call(CLEAR_PLAYERS_284, &mut g)?;
    call(TAKE_PLAYER1_UNITS, &mut g)?;
    call(CLEAR_EVENT_STATE, &mut g)?;
    set(ROUND, 0);
    set(PLAYER_COUNT, 3);
    set(EVENT_MODE, 8);
    set(LEAVE, 0);
    if !choose_cars(&mut g, call, 3, false, &[])? {
        return Ok(*entry);
    }
    unsafe { rally_rounds(&mut g, call, 3)? };
    end(&mut g, call)?;
    Ok(*entry)
}

fn test(t: crate::framework::SelfTest) -> (usize, usize) {
    let flow = Flow {
        address: 0x4828B0,
        len: 0x2CA,
        // pusha; call menu::begin_event
        entry: &[0x60, 0xE8, 0xF4, 0x02, 0xFF, 0xFF],
        calls: CALLS,
        fatal_jumps: &[],
        writes: &WRITES,
    };
    let mut rounds = crate::menu::single_rally_two::round_scripts(3, RESULTS_482E9B);
    rounds.push((
        "two rounds, the second's overall results' yes".into(),
        vec![
            (
                RACE,
                (0..6).map(|k| Answer::write(RACE_RESULT, k)).collect(),
            ),
            (SCREEN_483163, vec![Answer::yes()]),
        ],
    ));
    test_rounds(t, &flow, 3, rounds, &|r, c| unsafe {
        single_rally_three(r, c)
    })
}
