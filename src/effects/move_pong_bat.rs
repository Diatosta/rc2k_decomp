//! `effects::move_pong_bat`: the pong bat's place kept (`PongState.saved_bat[0]`,
//! `PongState.saved_bat[1]`), then `effects::pong_bat_back`. The keys' moves between (up
//! and down by `Devices.keys_now`, held to a limit) are jumped over: the bat never moves
//! (a slip kept as is).

use crate::effects::pong::Pong;
use crate::effects::pong_bat_back::pong_bat_back;

pub fn move_pong_bat(pong: &Pong) {
    let p = &pong.state;
    p.saved_bat[0].set(p.bat[0].get());
    p.saved_bat[1].set(p.bat[1].get());
    pong_bat_back(pong);
}
