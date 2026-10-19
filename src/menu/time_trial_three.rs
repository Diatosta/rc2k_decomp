//! `menu::time_trial_three` (0x48B54B): the time trial for three players in turn (event
//! mode 0x16). `g_eventMode` 3, `menu::begin_event`, `clear_players_284`,
//! `take_player1_units`; `dword_758714` 0, `dword_755984` 3, `dword_755988` 0x16,
//! `dword_755954` 3, `dword_75555D` 0, the eight dwords `dword_7587A8` .. `dword_7587C4`
//! 0; the three car choices (players 1 and 2's list made again before their car is
//! taken; left, it ends at once). Then each stage: the current player 0,
//! `menu::ready_players`, the four repairs, players 2 and 3's times (+0x88) 0x36F268 and
//! 0x36F650, the stage choice (a yes or `dword_75555D` ends it), `dword_755938` 1, the
//! service (its yes not looked at), the three races (`rounds::race_series`, ebx 3; a race
//! left, -2, goes to the review); unless all results are -1: the results page (a yes ends
//! it), the stage's record in edi, the three figures kept, `dword_755920` 0, the marks
//! cleared, each time entered when it qualifies, the best times page when one did (a yes
//! ends it); the review (eax 3, a yes asked). The end: `rounds::end`.
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
use crate::menu::time_trial_split::time_trial_start;

const CALLS: &[usize] = &[
    0xB, 0x10, 0x15, 0xAC, 0xB1, 0xC3, 0xD8, 0xDD, 0xF5, 0xFA, 0x10F, 0x114, 0x12C, 0x131, 0x140,
    0x14F, 0x15E, 0x16D, 0x17C, 0x1A1, 0x1CA, 0x1E3, 0x1F1, 0x1F6, 0x21A, 0x228, 0x22D, 0x254,
    0x262, 0x267, 0x282, 0x2C0, 0x2CF, 0x2DE, 0x2ED, 0x302, 0x310, 0x31F, 0x32D, 0x33C, 0x34A,
    0x358, 0x36B, 0x370, 0x3BD,
];

flow_hook!("menu::time_trial_three", 0x48B54B, time_trial_three, test);

/// `dword_7587B8` .. `dword_7587C4`, then `dword_7587A8` .. `dword_7587B4`, in the
/// original's order.
const CLEARED: [usize; 8] = [
    0x7587B8, 0x7587BC, 0x7587C0, 0x7587C4, 0x7587A8, 0x7587AC, 0x7587B0, 0x7587B4,
];

/// The rounds of the three and four players' time trials after the car choices:
/// `four` for `menu::time_trial_four`'s differences (no current player 0 before
/// `ready_players`, the service's yes looked at, no stage record in edi, the review
/// after `menu::show_in_game_options`).
pub unsafe fn time_trial_rounds(
    g: &mut Regs,
    call: Call,
    n: u32,
    four: bool,
) -> Result<(), Refused> {
    loop {
        if !four {
            set(CURRENT, 0);
        }
        call(READY_PLAYERS, g)?;
        repair_all(g, call)?;
        g.edi = player(2);
        put(g.edi + 0x88, TIME_2);
        g.edi = player(3);
        put(g.edi + 0x88, TIME_3);
        call(TIME_TRIAL_SELECT_STAGE, g)?;
        if yes() || get(LEAVE) == 1 {
            return Ok(());
        }
        set(SERVICE_DUE, 1);
        call(TIME_TRIAL_SERVICE, g)?;
        if four && yes() {
            return Ok(());
        }
        if let Some(all) = race_series(g, call, n, 3, false)?
            && all != u32::MAX
        {
            call(TIME_TRIAL_RESULTS_LOOP, g)?;
            if yes() {
                return Ok(());
            }
            if !four {
                stage_into(g)?;
            }
            keep_bests(g, call, n)?;
            set(ANY_QUALIFIED, 0);
            call(CLEAR_TIME_TRIAL_MARKS, g)?;
            qualify(g, call, 0, n)?;
            if !best_times(g, call)? {
                return Ok(());
            }
        }
        if review(g, call, if four { None } else { Some(n) }, true)? {
            return Ok(());
        }
    }
}

pub unsafe fn time_trial_three(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    players_there(4)?;
    let mut g = *entry;
    time_trial_start(&mut g, call)?;
    set(ROUND, 0);
    set(PLAYER_COUNT, 3);
    set(EVENT_MODE, 0x16);
    set(EVENT_TYPE, 3);
    set(LEAVE, 0);
    for v in CLEARED {
        set(v, 0);
    }
    if !choose_cars(&mut g, call, 3, true, &[1, 2])? {
        return Ok(*entry);
    }
    unsafe { time_trial_rounds(&mut g, call, 3, false)? };
    end(&mut g, call)?;
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
