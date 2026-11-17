//! `effects::move_pong_bat`: the pong bat's place kept (`PongState.saved_bat[0]`,
//! `PongState.saved_bat[1]`), moved up or down a step by player 1's steering (left up, right
//! down) and held between the walls, then it is put back where it was if it
//! now overlaps the ball. (The original jumps over the moves, so its bat never moves.)

use crate::effects::pong::{self, Pong};

/// `steer`: negative for left, positive for right, 0 for none.
pub fn move_pong_bat(pong: &Pong, effects: &crate::effects::owner::Effects, steer: i32) {
    let rules = pong::rules(pong, effects);
    pong::with_court(pong, |court| {
        court.move_bat(&rules, steer);
    });
}
