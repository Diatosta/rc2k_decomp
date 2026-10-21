//! The tests of the single rally's and the time trial's multi-player events (IDA,
//! 2026-10-19; each event a hook of its own, `menu::single_rally_split` .. `four`,
//! `menu::time_trial_one` .. `four`, converted 2026-10-21 to `rc2k_core::menu::rounds`).

use crate::framework::{Regs, SelfTest};
use crate::menu::event_flow::*;
use crate::menu::time_trial_qualifies::{ANY_QUALIFIED, QUALIFIES};

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
