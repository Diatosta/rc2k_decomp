//! `net::send_turn` (0x46415B): the own car's messages by its step count (the current
//! car's block +8, `unk_719FE0`): every fourth step (& 3 = 0) `net::send_inputs`, at & 0x1F
//! = 2 `net::send_position`, at & 0xFF = 1 `net::send_extra`.
//!
//! In:  nothing
//! Out: nothing (the original leaves eax the count & 0xFF or as the sends leave it; its one
//!      caller reads esi only after it: `lq -s 0x46415B`)
//!
//! See `net/mod.rs`. One caller, `net::race_step`.

use crate::errors::Refused;
use crate::net::{CAR, a, send_extra, send_inputs, send_position};
use rc2k_core::net::race::SENT;

/// The Rust version.
pub fn send_turn() -> Result<(), Refused> {
    let steps = a(CAR) + 8;
    let count = || unsafe { (steps as *const u32).read_volatile() };
    let sends: [fn() -> Result<(), Refused>; 3] = [
        send_inputs::send_inputs,
        send_position::send_position,
        send_extra::send_extra,
    ];
    // Converted: `rc2k_core::net::race::Kind::due`.
    for (kind, send) in SENT.into_iter().zip(sends) {
        if kind.due(count()).0 {
            send()?;
        }
    }
    Ok(())
}
