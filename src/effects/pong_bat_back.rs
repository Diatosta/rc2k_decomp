//! `effects::pong_bat_back`: the pong bat put back where it was before its move
//! (`PongState.saved_bat[0]`, `PongState.saved_bat[1]`) when it overlaps the ball
//! (`effects::pong`'s four compares).

use crate::effects::pong::{self, Pong};

pub fn pong_bat_back(pong: &Pong) {
    let (over, _) = pong::overlap(pong);
    if over {
        let p = &pong.state;
        p.bat[0].set(p.saved_bat[0].get());
        p.bat[1].set(p.saved_bat[1].get());
    }
}
