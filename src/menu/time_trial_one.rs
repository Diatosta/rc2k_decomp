//! `menu::time_trial_one` (0x48AE39): the time trial for one player (event mode 0x14).
//! `menu::clear_players_284`, `take_player1_units`; `dword_755984` 1, `dword_755988`
//! 0x14, `dword_755954` 3, `dword_75555D` 0, `g_eventMode` 3; `menu::begin_event`; the
//! car choice (`rounds::choose_cars`, the time trial's list; left, it ends at once). Then
//! each stage: the stage choice (`menu::time_trial_select_stage_loop`; a yes or
//! `dword_75555D` ends it), `begin_event`, `dword_755938` 1, the service
//! (`menu::time_trial_service`; a yes ends it), the race (`race::single_rally_race`, eax
//! 0, ebx 3); unless its result is -1: the current player 0, the result page
//! (`menu::stage_result_loop`; a yes ends it), the stage's record in edi
//! (`rounds::stage_into`), `menu::keep_stage_best`, `dword_755918` 0,
//! `menu::time_trial_qualifies`, and when it qualifies `clear_time_trial_marks`,
//! `enter_time_trial_best` and the best times page (`time_trial_best_times_loop`; a yes
//! ends it); the review (`rounds::review`, eax 1, a yes asked). The end: `rounds::end`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through the time trial's start menu (0x48BCDC's items). A call at +1: a
//! private copy (0x197 bytes, its 19 calls relocated) and a jump written by hand. Ungated
//! (a whole event). Refuses a missing player before anything, and what the callees and
//! `rounds::stage_into` refuse when they are reached. Shadow mode runs the original alone.
//! Self-test: `rounds::test_rounds`.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::event_flow::*;
use crate::menu::rounds::*;
use crate::menu::run_championship::SERVICE_DUE;
use crate::menu::time_trial_qualifies::QUALIFIES;

const CALLS: &[usize] = &[
    0x1, 0x6, 0x3D, 0x52, 0x57, 0x69, 0x6E, 0x8D, 0x9C, 0xB8, 0xD0, 0x104, 0x113, 0x121, 0x126,
    0x12B, 0x13E, 0x143, 0x190,
];

flow_hook!(
    "menu::time_trial_one",
    0x48AE39,
    time_trial_one,
    test,
    copy(0x197, CALLS)
);

pub unsafe fn time_trial_one(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    players_there(1)?;
    let mut g = *entry;
    call(CLEAR_PLAYERS_284, &mut g)?;
    call(TAKE_PLAYER1_UNITS, &mut g)?;
    set(PLAYER_COUNT, 1);
    set(EVENT_MODE, 0x14);
    set(EVENT_TYPE, 3);
    set(LEAVE, 0);
    set(MODE, 3);
    call(BEGIN_EVENT, &mut g)?;
    if !choose_cars(&mut g, call, 1, true, &[])? {
        return Ok(*entry);
    }
    loop {
        call(TIME_TRIAL_SELECT_STAGE, &mut g)?;
        if yes() || get(LEAVE) == 1 {
            break;
        }
        call(BEGIN_EVENT, &mut g)?;
        set(SERVICE_DUE, 1);
        call(TIME_TRIAL_SERVICE, &mut g)?;
        if yes() {
            break;
        }
        g.eax = 0;
        g.ebx = 3;
        call(RACE, &mut g)?;
        if get(RACE_RESULT) != u32::MAX {
            set(CURRENT, 0);
            call(STAGE_RESULT_LOOP, &mut g)?;
            if yes() {
                break;
            }
            stage_into(&mut g)?;
            call(KEEP_STAGE_BEST, &mut g)?;
            set(QUALIFIES, 0);
            call(TIME_TRIAL_QUALIFIES, &mut g)?;
            if get(QUALIFIES) == 1 {
                call(CLEAR_TIME_TRIAL_MARKS, &mut g)?;
                call(ENTER_TIME_TRIAL_BEST, &mut g)?;
                call(TIME_TRIAL_BEST_TIMES_LOOP, &mut g)?;
                if yes() {
                    break;
                }
            }
        }
        if review(&mut g, call, Some(1), true)? {
            break;
        }
    }
    end(&mut g, call)?;
    Ok(*entry)
}

/// The time trial's rounds' scripts for `n` players (`results` the results page).
pub fn time_trial_scripts(n: u32, results: usize) -> Vec<(String, Script)> {
    let race = |v: u32| Answer::write(RACE_RESULT, v);
    let q = || Answer::write(QUALIFIES, 1).and(crate::menu::time_trial_qualifies::ANY_QUALIFIED, 1);
    let mut out = crate::menu::single_rally_two::round_scripts(n, results);
    out.extend([
        (
            "the service's yes".to_string(),
            vec![(TIME_TRIAL_SERVICE, vec![Answer::yes()])],
        ),
        (
            "qualified, the best times page's yes".to_string(),
            vec![
                (RACE, (0..n).map(|k| race(k + 7)).collect()),
                (TIME_TRIAL_QUALIFIES, vec![q()]),
                (TIME_TRIAL_BEST_TIMES_LOOP, vec![Answer::yes()]),
            ],
        ),
        (
            "the last qualified, then the review ends".to_string(),
            vec![
                (RACE, (0..n).map(|k| race(k + 9)).collect()),
                (
                    TIME_TRIAL_QUALIFIES,
                    (0..n)
                        .map(|k| if k + 1 == n { q() } else { Answer::none() })
                        .collect(),
                ),
                (REVIEW_485F5E, vec![Answer::write(ENDED_569, 1)]),
            ],
        ),
    ]);
    out
}

fn test(t: crate::framework::SelfTest) -> (usize, usize) {
    if stage_record().is_err() {
        return t.skip("no stage tables for the chosen stage");
    }
    let flow = Flow {
        address: 0x48AE39,
        len: 0x197,
        // pusha; call menu::clear_players_284
        entry: &[0x60, 0xE8, 0x3A, 0xE8, 0x01, 0x00],
        calls: CALLS,
        fatal_jumps: &[],
        writes: &WRITES,
    };
    test_rounds(
        t,
        &flow,
        1,
        time_trial_scripts(1, STAGE_RESULT_LOOP),
        &|r, c| unsafe { time_trial_one(r, c) },
    )
}
