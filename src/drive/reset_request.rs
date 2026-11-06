//! `drive::reset_request` (0x43A0D3): the reset key held. Once the race is past 0x320
//! (`unk_719FEC`, by the sign): with the keys' bit 0x80 (`unk_71A440`), outside mode 2
//! (`dword_62389C`) and the car's speed below `flt_613828` (or unordered): a notice set up
//! on the first step (`unk_71A6F4` 0xCD, `unk_71A708` 0x19, `unk_71A704` 0x12C), the count
//! `unk_71A23C` + 1, at 100 the time penalty (0x2BC less `unk_71A1C0`) added to
//! `unk_71A060`, the notice cleared and `physics::reset_car`. Else a count under way is
//! cleared with the notice.
//!
//! In:  nothing
//! Out: ebx `unk_71A440` past the first test; at the reset as `physics::reset_car` leaves
//!      the registers (eax the penalty); the rest kept; no flags read
//!
//! Callers: `drive::car_controls`, `sub_460835`, `sub_460B2F`. Refuses what
//! `physics::reset_car` refuses.

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::flow::c0;
use crate::physics::tables_store::PHYSICS;
use crate::race::owner::race;
use crate::race::vars_store::RACE_VARS;
use rc2k_core::drive::controls::{
    RESET_NOTICE, reset_counts, reset_held, reset_penalty, reset_step,
};

fn clear_notice() {
    for cell in [
        &CAR.notice_last,
        &CAR.notice_id,
        &CAR.notice_time,
        &CAR.notice_shown,
    ] {
        cell.set(0);
    }
}

/// The Rust version, then `physics::reset_car`.
// Converted: `rc2k_core::drive::controls::{reset_counts, reset_held, reset_step, reset_penalty}`.
pub fn reset_request() -> Result<(), Refused> {
    let race = race();
    if !reset_counts(CAR.run_ticks.get()) {
        return Ok(());
    }
    if !reset_held(CAR.input_keys.get(), RACE_VARS.run_kind.get(), || {
        c0(CAR.speed.bits(), PHYSICS.reset_speed_limit.bits())
    }) {
        if CAR.reset_key_count.get() != 0 {
            clear_notice();
            CAR.reset_key_count.set(0);
        }
        return Ok(());
    }
    let (first, count, reset) = reset_step(CAR.reset_key_count.get());
    if first {
        CAR.notice_id.set(RESET_NOTICE);
        CAR.notice_time_b.set(0x19);
        CAR.notice_time.set(0x12C);
    }
    CAR.reset_key_count.set(count);
    if !reset {
        return Ok(());
    }
    CAR.stage_time.set(
        CAR.stage_time
            .get()
            .wrapping_add(reset_penalty(CAR.stuck_count.get())),
    );
    clear_notice();
    { crate::physics::reset_car::reset_car(race) }?;
    Ok(())
}
