//! `physics::pace_notes_step` (0x4417EC): the work car's pace notes, each physics step.
//! Only with `unk_71A080`, `unk_71A07C`, `unk_71A078`, `g_holdCarRevs`, `dword_71C3E0`,
//! `unk_719FF4`, `unk_71A22C` all 0, `unk_71A234` 1 and `unk_719FEC` 1000 or more
//! (unsigned). Then the checkpoints ahead: `g_carSpeed` x `flt_614C78` / `flt_5C5D50`
//! (`fistp`) into `dword_62400C`; ebx the car's checkpoint (`unk_71A06C`) plus that when
//! not negative, 0 when at or past the count (by the sign); past the last called
//! (`unk_71A074`, unsigned): more than 16 past moves it up to ebx first, then every
//! checkpoint after it up to ebx (at least one) has its group called
//! (`physics::queue_pace_note`, edx the checkpoint, inside `pusha` / `popa`), and
//! `unk_71A074` = ebx. Then (unless it left at the start) `physics::expire_car_events`.
//!
//! In:  nothing
//! Out: the start's gates closed: every register kept; else ebx and edx as above (edx not
//!      set when nothing was called) and what `expire_car_events` leaves; no flags read
//!
//! Caller: `physics::car_physics_step`, which calls the Rust version directly. Its calls
//! by `physics/queue_pace_note.rs`' places (5 the group, 6 the expiry).
//! Refuses what its calls refuse. Shadow mode runs the original alone (the calls queue
//! sounds and car events). Self-test: a copy whose two calls go to stand-ins that note
//! them, against the Rust version with the same, on the game's variables (put back): each
//! gate, speeds forward and back, checkpoints about the count and the last called.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use rc2k_core::physics::pace_calls::{call_target, calling, calls};

/// The gates: (variable, the value it must have); the last is "1000 or more".
const ZERO: [usize; 7] = [
    0x71A080, 0x71A07C, 0x71A078, 0x5BEC94, 0x71C3E0, 0x719FF4, 0x71A22C,
];
const ONE: usize = 0x71A234;
const TIME: usize = 0x719FEC;
const SPEED: usize = 0x71A184; // g_carSpeed
const SCALE: usize = 0x614C78; // flt_614C78
const DIVISOR: usize = 0x5C5D50; // flt_5C5D50
const AHEAD: usize = 0x62400C; // dword_62400C
const CHECKPOINT: usize = 0x71A06C;
const LAST_CALLED: usize = 0x71A074;
const COUNT: usize = 0x623640; // g_checkpointCount
const ONE_F32: u32 = 0x3F80_0000;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::pace_calls::{calling, call_target, calls}`.
pub fn pace_notes_step() -> Result<(), Refused> {
    if !calling(ZERO.map(g), g(ONE), g(TIME)) {
        return Ok(());
    }
    // `fld; fmul; fdiv; fistp` (the extra division by 1.0 is exact).
    let ahead = crate::math::x87::mul_div_div_to_int(g(SPEED), g(SCALE), g(DIVISOR), ONE_F32);
    p(AHEAD, ahead as u32);
    let target = call_target(g(CHECKPOINT), ahead, g(COUNT));
    if let Some((last, _)) = calls(target, g(LAST_CALLED)) {
        p(LAST_CALLED, last);
        let mut checkpoint = last;
        loop {
            checkpoint = checkpoint.wrapping_add(1);
            crate::physics::queue_pace_note::queue_pace_note(checkpoint)?;
            if checkpoint >= target {
                break;
            }
        }
        p(LAST_CALLED, target);
    }
    unsafe {
        crate::physics::expire_car_events::expire_car_events_at(
            &crate::physics::queue_car_event::QueuePlaces::game(),
        )
    }?;
    Ok(())
}
