//! `menu::time_trial_two` (0x48B23D): the time trial for two players in turn (event mode
//! 0x15). As `menu::time_trial_split`'s start (`dword_755988` 0x15). Then each stage: the
//! current player 0, `menu::ready_players`, the four repairs, players 2 and 3's times
//! (+0x88, through `dword_75598C` 2 and 3) 0x36F268 and 0x36F650, the stage choice (a yes
//! or `dword_75555D` ends it), `dword_755938` 1, the service (its yes not looked at), the
//! two races (`rounds::race_series`, ebx 3; a race left, -2, goes to the review); when
//! the results are not all -1: the results page (`menu::time_trial_results_loop`; a yes
//! ends it), the stage's record in edi, both figures kept, `dword_755920` 0, the marks
//! cleared, player 0's time entered when it qualifies; then (also when all were -1, the
//! original's jump goes there) player 1's; the best times page when one qualified (a yes
//! ends it); the review (eax 2, a yes asked). The end: `rounds::end`.
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
use crate::menu::time_trial_split::{CLEARED_TWO, time_trial_start};

const CALLS: &[usize] = &[
    0xB, 0x10, 0x15, 0x7A, 0x7F, 0x91, 0xA6, 0xAB, 0xBD, 0xCC, 0xDB, 0xEA, 0xF9, 0x108, 0x151,
    0x17A, 0x193, 0x1A1, 0x1A6, 0x1CA, 0x1D8, 0x1DD, 0x1F8, 0x236, 0x245, 0x254, 0x269, 0x277,
    0x286, 0x294, 0x2A2, 0x2B5, 0x2BA, 0x307,
];

flow_hook!("menu::time_trial_two", 0x48B23D, time_trial_two, test);

pub unsafe fn time_trial_two(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    players_there(4)?;
    let mut g = *entry;
    time_trial_start(&mut g, call)?;
    set(PLAYER_COUNT, 2);
    set(EVENT_MODE, 0x15);
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
        repair_all(&mut g, call)?;
        for (k, time) in [(2, TIME_2), (3, TIME_3)] {
            set(CURRENT, k);
            g.edi = player(get(CURRENT));
            put(g.edi + 0x88, time);
        }
        call(TIME_TRIAL_SELECT_STAGE, &mut g)?;
        if yes() || get(LEAVE) == 1 {
            break;
        }
        set(SERVICE_DUE, 1);
        call(TIME_TRIAL_SERVICE, &mut g)?;
        if let Some(all) = race_series(&mut g, call, 2, 3, false)? {
            let first = if all != u32::MAX {
                call(TIME_TRIAL_RESULTS_LOOP, &mut g)?;
                if yes() {
                    break;
                }
                stage_into(&mut g)?;
                keep_bests(&mut g, call, 2)?;
                set(ANY_QUALIFIED, 0);
                call(CLEAR_TIME_TRIAL_MARKS, &mut g)?;
                0
            } else {
                1
            };
            qualify(&mut g, call, first, 2)?;
            if !best_times(&mut g, call)? {
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

fn test(t: crate::framework::SelfTest) -> (usize, usize) {
    if stage_record().is_err() {
        return t.skip("no stage tables for the chosen stage");
    }
    let flow = Flow {
        address: 0x48B23D,
        len: 0x30E,
        // pusha; mov g_eventMode, 3
        entry: &[
            0x60, 0xC7, 0x05, 0xD0, 0x56, 0x75, 0x00, 0x03, 0x00, 0x00, 0x00,
        ],
        calls: CALLS,
        fatal_jumps: &[],
        writes: &WRITES,
    };
    let rounds = crate::menu::time_trial_one::time_trial_scripts(2, TIME_TRIAL_RESULTS_LOOP);
    test_rounds(t, &flow, 2, rounds, &|r, c| unsafe { time_trial_two(r, c) })
}
