//! `net::apply_message` (0x4644EC): one message received from player ebp put into that
//! player's car block (`g_carBlocks[ebp]`), after logging the own player, the step, the
//! message's first byte, "rcvd fm plyr" and ebp. Players 8 and up: nothing more.
//! - 'JGSM': the four input dwords into +0xFD0 .. +0xFDC;
//! - 'XGSM': nine dwords into +0x64 .. +0x84;
//! - 'PGSM': the place into +0xFC0 .. +0xFC8 (where the car is to be), the other 37
//!   dwords where `net::send_position` took them from. Then, after the car's first 1000
//!   steps (+8; `js` on +8 - 1000) and when the place received is 10 or more away from the
//!   car's (+0x134 .. +0x13C, whole numbers: `fild` / `fisub`, `fsqrt`, `flt_5C5D54`) the
//!   car is put there; else +0xFC0 .. +0xFC8 become a 32nd of the way (`sar 5`) and
//!   +0xFCC 32, for `net::follow_remote` to glide it;
//! - others: nothing.
//!
//! In:  `player` (ebp), `message` (ebx: its first dword the tag)
//! Out: nothing (the original leaves eax, ecx, edi as it goes; its one caller has it inside
//!      `pusha` .. `popa`)
//!
//! See `net/mod.rs`. One caller, `net::apply_received`. The distance with the original's
//! x87 instructions.

use crate::framework::{game_read, rva};
use crate::logfile::{log_int::log_int, log_write::log_write};
use crate::net::store::NET;
use crate::net::{STEP, a, g};
use rc2k_core::net::race::{self, Arrival, Kind};

/// g_carBlocks: the eight cars' blocks.
pub const BLOCKS: usize = 0x71B3B8;
const TEXT: usize = 0x4DE0EE; // "rcvd fm plyr"
const NEAR: usize = 0x5C5D54; // flt_5C5D54: 10.0

/// The Rust version.
pub fn apply_message(player: u32, message: *const u8) {
    let rd = |at: u32| unsafe { (at as *const u32).read_unaligned() };
    let wr = |at: u32, v: u32| unsafe { (at as *mut u32).write_unaligned(v) };
    let message = message as u32;
    log_int(NET.player.get() as i32, false, 1);
    log_int(g(STEP) as i32, false, 1);
    log_int(unsafe { (message as *const u8).read() } as i32, false, 1);
    unsafe { log_write(a(TEXT) as *const u8, false, 1) };
    log_int(player as i32, true, 1);
    if player >= 8 {
        return;
    }
    let block = rd(a(BLOCKS) + 4 * player);
    // Converted: `rc2k_core::net::race` (`Kind`, `arrival`; the distance's test in `f64`,
    // exact for the decision).
    let Some(kind) = Kind::of(rd(message)) else {
        return;
    };
    for n in 0..kind.fields().len() as u32 {
        wr(
            block + kind.destination(n as usize),
            rd(message + 4 + 4 * n),
        );
    }
    if kind != Kind::Position {
        return;
    }
    let three = |at: u32| [rd(block + at), rd(block + at + 4), rd(block + at + 8)];
    let (wanted, now) = (three(race::WANTED), three(race::PLACE));
    let steps = rd(block + race::STEPS);
    let near = f32::from_bits(unsafe { game_read(rva(NEAR)) });
    match race::arrival(wanted, now, steps, near) {
        Arrival::Jump => {
            for (k, v) in (0..).zip(wanted) {
                wr(block + race::PLACE + 4 * k, v);
            }
        }
        Arrival::Glide(step) => {
            for (k, v) in (0..).zip(step) {
                wr(block + race::WANTED + 4 * k, v);
            }
            wr(block + race::GLIDE_STEPS, race::GLIDE);
        }
    }
}
