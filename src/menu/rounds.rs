//! What the single rally's and the time trial's multi-player events share (IDA,
//! 2026-10-19): the players' car choices, the series of races one player after another,
//! the time trial's bests, the end. Each event is a hook of its own
//! (`menu::single_rally_split` .. `four`, `menu::time_trial_one` .. `four`) written out as
//! its original is, with these pieces.

use crate::errors::Refused;
use crate::framework::{Regs, SelfTest};
use crate::menu::championship_welcome::addr;
use crate::menu::event_flow::*;
use crate::menu::time_trial_qualifies::{ANY_QUALIFIED, QUALIFIES};

/// The single rally's car list: `off_759DCA` with `dword_7554D8` 1, else `off_75A042`;
/// `off_75A412` with cheat bit 1 (the original's `bt`).
pub fn rally_car_list(g: &mut Regs) {
    g.esi = addr(if get(CAR_LIST_KIND) == 1 {
        CARS_FIRST
    } else {
        CARS_SECOND
    });
    let on = get(CHEATS) & 2 != 0;
    if on {
        g.esi = addr(CARS_CHEAT);
    }
    g.set_carry(on);
}

/// The time trial's car list: `off_75A042`, then `menu::cheat_car_list`.
pub fn time_trial_car_list(g: &mut Regs, call: Call) -> Result<(), Refused> {
    g.esi = addr(CARS_SECOND);
    call(CHEAT_CAR_LIST, g)
}

/// The car choices of players 0 .. n (`dword_75598C` each): the list, the choice
/// (`menu::select_car_loop`), and unless it was left (`dword_758806` 1: false) the car
/// taken (`menu::take_shown_car`). `list_again[k]`: the time trial's list is made again
/// before player k's car is taken (two of the originals do).
pub fn choose_cars(
    g: &mut Regs,
    call: Call,
    n: u32,
    time_trial: bool,
    list_again: &[u32],
) -> Result<bool, Refused> {
    for k in 0..n {
        set(CURRENT, k);
        if time_trial {
            time_trial_car_list(g, call)?;
        } else {
            rally_car_list(g);
        }
        call(SELECT_CAR_LOOP, g)?;
        if get(CAR_LEFT) == 1 {
            return Ok(false);
        }
        if list_again.contains(&k) {
            time_trial_car_list(g, call)?;
        }
        call(TAKE_SHOWN_CAR, g)?;
    }
    Ok(true)
}

/// The four players' repairs (`dword_75598C` 0 .. 3, `service::repair_car`).
pub fn repair_all(g: &mut Regs, call: Call) -> Result<(), Refused> {
    for k in 0..4 {
        set(CURRENT, k);
        call(REPAIR_CAR, g)?;
    }
    Ok(())
}

/// Players 2 and 3's times (+0x88) for the time trial (3 597 928 and 3 602 000 ms).
pub const TIME_2: u32 = 0x36F268;
pub const TIME_3: u32 = 0x36F650;

/// The races of players 0 .. n, one after another (`race::single_rally_race`, eax 1 but
/// for the last, ebx `kind`; the single rally repairs each car first): a result
/// (`dword_75B028`) of -2 resets the players' times (`menu::reset_player_times`) and sets
/// the front end up again (`menu::reload_front_end`): None. Else the results and-ed
/// together (eax, as the original keeps them between the races).
pub fn race_series(
    g: &mut Regs,
    call: Call,
    n: u32,
    kind: u32,
    repair: bool,
) -> Result<Option<u32>, Refused> {
    let mut all = 0;
    for k in 0..n {
        set(CURRENT, k);
        if repair {
            call(REPAIR_CAR, g)?;
        }
        g.eax = u32::from(k + 1 < n);
        g.ebx = kind;
        call(RACE, g)?;
        if get(RACE_RESULT) == 0xFFFF_FFFE {
            call(RESET_PLAYER_TIMES, g)?;
            call(RELOAD_FRONT_END, g)?;
            if k > 0 {
                g.eax = all;
            }
            return Ok(None);
        }
        all = if k == 0 {
            get(RACE_RESULT)
        } else {
            all & get(RACE_RESULT)
        };
        g.eax = all;
    }
    Ok(Some(all))
}

/// The chosen stage's record into edi as the time trial's originals find it (eax the
/// number x 0xA4, ebx the sub).
pub fn stage_into(g: &mut Regs) -> Result<(), Refused> {
    g.edi = stage_record()?;
    g.ebx = get(STAGE_SUB);
    g.eax = get(STAGE_NUMBER).wrapping_mul(STAGE_RECORD);
    Ok(())
}

/// Each of players 0 .. n's figure kept for the stage (`menu::keep_stage_best`).
pub fn keep_bests(g: &mut Regs, call: Call, n: u32) -> Result<(), Refused> {
    for k in 0..n {
        set(CURRENT, k);
        call(KEEP_STAGE_BEST, g)?;
    }
    Ok(())
}

/// Each of players `from` .. n: whether the time qualifies (`menu::time_trial_qualifies`;
/// for player 0 ecx `dword_755984` first), and if it does, the time entered
/// (`menu::enter_time_trial_best`).
pub fn qualify(g: &mut Regs, call: Call, from: u32, n: u32) -> Result<(), Refused> {
    for k in from..n {
        set(CURRENT, k);
        if k == 0 {
            g.ecx = get(PLAYER_COUNT);
        }
        call(TIME_TRIAL_QUALIFIES, g)?;
        if get(QUALIFIES) == 1 {
            call(ENTER_TIME_TRIAL_BEST, g)?;
        }
    }
    Ok(())
}

/// The best times page when one qualified (`dword_755920`): false after a yes on it.
pub fn best_times(g: &mut Regs, call: Call) -> Result<bool, Refused> {
    if get(ANY_QUALIFIED) == 1 {
        call(TIME_TRIAL_BEST_TIMES_LOOP, g)?;
        if yes() {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The review (`menu::hide_in_game_options` with eax `hide`, `review_options_485f5e_loop`):
/// true when it ends the event (a yes when `ask` says so, `dword_755569`, `dword_75555D`).
pub fn review(g: &mut Regs, call: Call, hide: Option<u32>, ask: bool) -> Result<bool, Refused> {
    match hide {
        Some(eax) => {
            g.eax = eax;
            call(HIDE_IN_GAME_OPTIONS, g)?;
        }
        None => call(SHOW_IN_GAME_OPTIONS, g)?,
    }
    call(REVIEW_485F5E, g)?;
    Ok((ask && yes()) || get(ENDED_569) == 1 || get(LEAVE) == 1)
}

/// The end: `dword_755980` 1, `dword_75555D`, `dword_755569`, `dword_755565` 0,
/// `menu::show_in_game_options`.
pub fn end(g: &mut Regs, call: Call) -> Result<(), Refused> {
    set(DONE, 1);
    set(LEAVE, 0);
    set(ENDED_569, 0);
    set(ENDED_565, 0);
    call(SHOW_IN_GAME_OPTIONS, g)
}

/// What these events write, for their tests.
pub const WRITES: [usize; 23] = [
    PLAYER_COUNT,
    EVENT_MODE,
    EVENT_TYPE,
    MODE,
    LEAVE,
    CURRENT,
    CAR_LEFT,
    DONE,
    ENDED_569,
    ENDED_565,
    ROUND,
    QUALIFIES,
    ANY_QUALIFIED,
    crate::menu::run_championship::SERVICE_DUE,
    0x7587A8,
    0x7587AC,
    0x7587B0,
    0x7587B4,
    0x7587B8,
    0x7587BC,
    0x7587C0,
    0x7587C4,
    RACE_RESULT_2,
];

/// A rounds event's test: the car choice left at each player and none, then rounds by
/// script: each round's answers (the stage choice, the races' results, the pages).
pub fn test_rounds(
    t: SelfTest,
    flow: &Flow,
    n: u32,
    rounds: Vec<(String, Script)>,
    rust: Rust,
) -> (usize, usize) {
    if (0..4).any(|k| player(k) == 0) {
        return t.skip("a player missing from g_players");
    }
    let prepare = || {
        for v in [LEAVE, ENDED_569, ANSWER, CAR_LEFT, ROUND] {
            set(v, 0);
        }
        set(RACE_RESULT, 0);
        set(RACE_RESULT_2, 0);
    };
    let mut cases = Vec::new();
    for left in 0..n as usize {
        cases.push(Case {
            what: format!("the car choice left at {left}"),
            prepare: &prepare,
            script: vec![(SELECT_CAR_LOOP, car_choices(n as usize, left))],
            before: Regs::SENTINEL,
        });
    }
    for (what, script) in rounds {
        cases.push(Case {
            what,
            prepare: &prepare,
            script,
            before: Regs::SENTINEL,
        });
    }
    test_flow(t, flow, &[], &cases, rust)
}

/// Race results for a script: each race's `dword_75B028` (and `dword_75B098`).
pub fn results(r: &[u32]) -> Vec<Answer> {
    r.iter().map(|&v| Answer::write(RACE_RESULT, v)).collect()
}
