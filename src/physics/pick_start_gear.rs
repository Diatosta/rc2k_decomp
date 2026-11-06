//! `physics::pick_start_gear` (0x43C8AC): from neutral, the gear to start in.
//! `physics::copy_car_word_8`; the engine speed kept in `dword_614144`;
//! `physics::revs_from_speed`; `dword_614140` 0 and `flt_61413C` 100.0; then gears 1 .. 6
//! (ebx) in turn: one `physics::gear_bit` does not refuse is set, `revs_from_speed`, the
//! engine speed over its top (+0x1DC) stored; the first below `flt_5CB958` is taken: set,
//! `revs_from_speed`, the engine speed moved back toward the kept one by `flt_613FE0`
//! ((kept - e) x k + e), `physics::wheels_from_revs`, and the change settled (+0x1D0,
//! the hold 20, +0x238 and +0x23C 1). None taken: the last tried stays.
//!
//! In:  nothing
//! Out: the registers as the calls leave them (eax, ebx, ecx); no flags read (the
//!      callers return)
//!
//! Callers: `physics::manual_gearbox`, `physics::auto_gearbox`. Refuses what the calls
//! refuse; what was written is put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87_run::sub_mul_add;
use crate::physics::car_store::CAR;
use crate::physics::copy_car_word_8::copy_car_word_8;
use crate::physics::gear_store::GEAR;
use crate::physics::gearbox::*;
use crate::physics::tables_store::PHYSICS;
use crate::race::owner_course::Course;
use rc2k_core::physics::gear_shifts as shifts;

/// The steps on `r`.
// Converted: `rc2k_core::physics::gear_shifts::pick_start` (the engine's sums and compares
// stay x87).
pub fn pick_start_gear(course: &Course) -> Result<(), Refused> {
    copy_car_word_8();
    PHYSICS.start_gear_figures[2].set(CAR.engine_speed.bits());
    revs_speed()?;
    PHYSICS.start_gear_figures[1].set(0);
    PHYSICS.start_gear_figures[0].set(x87::loaded(course.numbers.hundred.bits()));
    let taken = shifts::pick_start(|gear| {
        if bit(gear)? {
            return Ok::<_, Refused>(false);
        }
        CAR.gear.set(gear);
        revs_speed()?;
        CAR.engine_speed.set_bits(over_top());
        Ok(below(CAR.engine_speed.bits(), GEAR.shift_up_point.bits()))
    })?;
    let Some(gear) = taken else {
        return Ok(());
    };
    CAR.gear.set(gear);
    revs_speed()?;
    CAR.engine_speed.set_bits(sub_mul_add(
        PHYSICS.start_gear_figures[2].get(),
        CAR.engine_speed.bits(),
        PHYSICS.start_gear_share.bits(),
        CAR.engine_speed.bits(),
    ));
    wheels()?;
    settle();
    Ok(())
}
