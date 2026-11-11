//! `effects::move_pong_bat` (0x4B82B0): the pong bat's place kept (`PongState.saved_bat[0]`,
//! `PongState.saved_bat[1]`), then `effects::pong_bat_back`. The keys' moves between (up with
//! `Devices.keys_now` and `Devices.keys_now`, down with `Devices.keys_now`, none with `Devices.keys_now`, held
//! to `flt_7691E6` .. ) are jumped over by the original's two `jmp short`s: the bat
//! never moves (section 16).
//!
//! Caller: `effects::run_pong`. The call is its Rust version.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::effects::pong::Pong;
use crate::effects::pong_bat_back::pong_bat_back;

pub fn move_pong_bat(pong: &Pong) {
    let p = &pong.state;
    p.saved_bat[0].set(p.bat[0].bits());
    p.saved_bat[1].set(p.bat[1].bits());
    pong_bat_back(pong);
}
