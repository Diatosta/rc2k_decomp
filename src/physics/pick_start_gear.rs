//! `physics::pick_start_gear` (0x43C8AC): from neutral, the gear to start in.
//! `physics::copy_step_clock_to_mark`; the engine speed kept in `dword_614144`;
//! `physics::revs_from_speed`; `dword_614140` 0 and `flt_61413C` 100.0; then gears 1 .. 6
//! (ebx) in turn: one `physics::gear_bit` does not refuse is set, `revs_from_speed`, the
//! engine speed over its top (+0x1DC) stored; the first below `flt_5CB958` is taken: set,
//! `revs_from_speed`, the engine speed moved back toward the kept one by `flt_613FE0`
//! ((kept - e) x k + e), `physics::wheels_from_revs`, and the change settled (+0x1D0,
//! the hold 20, +0x238 and +0x23C 1). None taken: the last tried stays.
//!
//! Callers: `physics::manual_gearbox`, `physics::auto_gearbox`. Refuses what the calls
//! refuse; what was written is put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87_run::sub_mul_add;
use crate::physics::gear_store::GEAR;
use crate::physics::gearbox::*;
use crate::race::owner::Race;
use rc2k_core::physics::gear_shifts as shifts;

/// The steps on `r`.
// Converted: `rc2k_core::physics::gear_shifts::pick_start` (the engine's sums and compares
// stay x87).
pub fn pick_start_gear(race: &Race) -> Result<(), Refused> {
    race.shared.physics_tables.start_gear_figures[2].set(race.shared.car.engine_speed.bits());
    revs_speed(&race.shared)?;
    race.shared.physics_tables.start_gear_figures[1].set(0);
    race.shared.physics_tables.start_gear_figures[0]
        .set(x87::loaded(race.course.numbers.hundred.bits()));
    let taken = shifts::pick_start(|gear| {
        if bit(&race.shared, gear)? {
            return Ok::<_, Refused>(false);
        }
        race.shared.car.gear.set(gear);
        revs_speed(&race.shared)?;
        race.shared.car.engine_speed.set_bits(over_top(race));
        Ok(below(
            race.shared.car.engine_speed.bits(),
            GEAR.shift_up_point.bits(),
        ))
    })?;
    let Some(gear) = taken else {
        return Ok(());
    };
    race.shared.car.gear.set(gear);
    revs_speed(&race.shared)?;
    race.shared.car.engine_speed.set_bits(sub_mul_add(
        race.shared.physics_tables.start_gear_figures[2].get(),
        race.shared.car.engine_speed.bits(),
        race.shared.physics_tables.start_gear_share.bits(),
        race.shared.car.engine_speed.bits(),
    ));
    wheels(&race.shared)?;
    settle(race);
    Ok(())
}
