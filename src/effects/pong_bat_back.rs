//! `effects::pong_bat_back` (0x4B8215): the pong bat put back where it was before its move
//! (`dword_7686AC`, `dword_7686B0`) when it overlaps the ball (`effects::pong`'s four
//! compares).
//!
//! Caller: `effects::move_pong_bat`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::effects::pong::{self, Pong};

/// The Rust version (see the header).
pub fn pong_bat_back(pong: &Pong) {
    let (over, _) = pong::overlap(pong);
    if over {
        let p = &pong.state;
        p.bat[0].set_bits(p.saved_bat[0].get());
        p.bat[1].set_bits(p.saved_bat[1].get());
    }
}
