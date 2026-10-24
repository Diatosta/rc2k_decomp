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
//! Callers: `physics::manual_gearbox`, `physics::auto_gearbox`.
//! Refuses what the calls refuse; what was written is put back.
//! Self-test: `physics/gearbox.rs` (`test_gearbox`), gears made 0 first.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87_run::sub_mul_add;
use crate::physics::copy_car_word_8::copy_car_word_8;
use crate::physics::gearbox::*;
use rc2k_core::physics::gear_shifts as shifts;

/// The steps on `r`.
// Converted: `rc2k_core::physics::gear_shifts::pick_start` (the engine's sums and compares
// stay x87).
pub fn pick_start_gear() -> Result<(), Refused> {
    unsafe { copy_car_word_8() };
    p(0x614144, g(ENGINE));
    revs_speed()?;
    p(0x614140, 0);
    p(0x61413C, x87::loaded(g(0x5C5D14)));
    let taken = shifts::pick_start(|gear| {
        if bit(gear)? {
            return Ok::<_, Refused>(false);
        }
        p(GEAR, gear);
        revs_speed()?;
        p(ENGINE, over_top());
        Ok(below(g(ENGINE), g(0x5CB958)))
    })?;
    let Some(gear) = taken else {
        return Ok(());
    };
    p(GEAR, gear);
    revs_speed()?;
    p(
        ENGINE,
        sub_mul_add(g(0x614144), g(ENGINE), g(0x613FE0), g(ENGINE)),
    );
    wheels()?;
    settle();
    Ok(())
}
