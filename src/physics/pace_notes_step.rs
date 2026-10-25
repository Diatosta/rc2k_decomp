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
//! Caller: `physics::car_physics_step`, which calls the Rust version directly. Its calls by
//! `physics/queue_pace_note.rs`' places (5 the group, 6 the expiry). Refuses what its calls
//! refuse.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, rva};
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::numbers_store::NUMBERS;
use crate::race::vars_store::RACE_VARS;
use crate::scene::draw_store::DRAW;
use rc2k_core::physics::pace_calls::{call_target, calling, calls};

/// The gates: (variable, the value it must have); the last is "1000 or more".
const ZERO: [usize; 7] = [
    0x71A080, 0x71A07C, 0x71A078, 0x5BEC94, 0x71C3E0, 0x719FF4, 0x71A22C,
];
const ONE_F32: u32 = 0x3F80_0000;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::pace_calls::{calling, call_target, calls}`.
pub fn pace_notes_step() -> Result<(), Refused> {
    if !calling(ZERO.map(g), CAR.driver_kind.get(), CAR.run_ticks.get()) {
        return Ok(());
    }
    // `fld; fmul; fdiv; fistp` (the extra division by 1.0 is exact).
    let ahead = crate::math::x87::mul_div_div_to_int(
        CAR.local_velocity[2].bits(),
        PHYSICS.pace_scale.bits(),
        NUMBERS.eight.bits(),
        ONE_F32,
    );
    DRAW.scratch.set(ahead as u32);
    let target = call_target(
        CAR.reset_checkpoint.get(),
        ahead,
        RACE_VARS.checkpoint_count.get(),
    );
    if let Some((last, _)) = calls(target, CAR.last_pace_call.get()) {
        CAR.last_pace_call.set(last);
        let mut checkpoint = last;
        loop {
            checkpoint = checkpoint.wrapping_add(1);
            crate::physics::queue_pace_note::queue_pace_note(checkpoint)?;
            if checkpoint >= target {
                break;
            }
        }
        CAR.last_pace_call.set(target);
    }
    unsafe {
        crate::physics::expire_car_events::expire_car_events_at(
            &crate::physics::queue_car_event::QueuePlaces::game(),
        )
    }?;
    Ok(())
}
