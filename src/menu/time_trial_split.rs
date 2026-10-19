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
//! cleared, each player's time entered when it qualifies (`rounds::qualify`); the best
//! times page when one did (a yes ends it); the review (`rounds::review`, eax 2; its yes
//! not looked at). The end: `rounds::end`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Reached through the time trial's start menu. The first five bytes hold no jump target
//! or call. Ungated (a whole event). Refuses a missing player before anything, and what
//! the callees and `rounds::stage_into` refuse when they are reached. Shadow mode runs the
//! original alone. Self-test: `rounds::test_rounds`.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::event_flow::*;
use crate::menu::rounds::*;
use crate::menu::run_championship::SERVICE_DUE;
use crate::menu::time_trial_qualifies::ANY_QUALIFIED;

const CALLS: &[usize] = &[
    0xB, 0x10, 0x15, 0x7A, 0x7F, 0x91, 0xA6, 0xAB, 0xBD, 0xCC, 0xDB, 0xEA, 0x10F, 0x138, 0x147,
    0x160, 0x19E, 0x1AD, 0x1BC, 0x1D1, 0x1DF, 0x1EE, 0x1FC, 0x20A, 0x21D, 0x222, 0x266,
];

flow_hook!("menu::time_trial_split", 0x48AFD0, time_trial_split, test);

/// The four dwords `dword_7587B8`, `dword_7587BC`, `dword_7587A8`, `dword_7587AC`.
pub const CLEARED_TWO: [usize; 4] = [0x7587B8, 0x7587BC, 0x7587A8, 0x7587AC];

/// The time trial's start for two to four players: `g_eventMode` 3, `begin_event`,
/// `clear_players_284`, `take_player1_units`.
pub fn time_trial_start(g: &mut Regs, call: Call) -> Result<(), Refused> {
    set(MODE, 3);
    call(BEGIN_EVENT, g)?;
    call(CLEAR_PLAYERS_284, g)?;
    call(TAKE_PLAYER1_UNITS, g)
}

pub unsafe fn time_trial_split(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    players_there(4)?;
    let mut g = *entry;
    time_trial_start(&mut g, call)?;
    set(PLAYER_COUNT, 2);
    set(EVENT_MODE, 4);
    set(EVENT_TYPE, 3);
    set(LEAVE, 0);
    for v in CLEARED_TWO {
        set(v, 0);
    }
    if !choose_cars(&mut g, call, 2, true, &[])? {
        return Ok(*entry);
    }
    loop {
        set(CURRENT, 0);
        call(READY_PLAYERS, &mut g)?;
        for k in 0..2 {
            set(CURRENT, k);
            call(REPAIR_CAR, &mut g)?;
        }
        g.edi = player(2);
        put(g.edi + 0x88, TIME_2);
        g.edi = player(3);
        put(g.edi + 0x88, TIME_3);
        call(TIME_TRIAL_SELECT_STAGE, &mut g)?;
        if yes() || get(LEAVE) == 1 {
            break;
        }
        set(SERVICE_DUE, 1);
        call(TIME_TRIAL_SERVICE, &mut g)?;
        g.eax = 0;
        g.ebx = 3;
        call(RACE, &mut g)?;
        g.eax = get(RACE_RESULT) & get(RACE_RESULT_2);
        if g.eax != u32::MAX {
            call(TIME_TRIAL_RESULTS_LOOP, &mut g)?;
            if yes() {
                break;
            }
            stage_into(&mut g)?;
            keep_bests(&mut g, call, 2)?;
            set(ANY_QUALIFIED, 0);
            call(CLEAR_TIME_TRIAL_MARKS, &mut g)?;
            qualify(&mut g, call, 0, 2)?;
        }
        if !best_times(&mut g, call)? {
            break;
        }
        if review(&mut g, call, Some(2), false)? {
            break;
        }
    }
    end(&mut g, call)?;
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
