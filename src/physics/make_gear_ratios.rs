//! `physics::make_gear_ratios` (0x4219E6): the car's gear ratios (`g_gearRatios`, by gear,
//! `physics/gearing.rs`) from its setup: the top gear's (`g_gearRatios[g_gearCount]`) and
//! the first's (`flt_6140C4`) each lo + (hi - lo) `flt_5C5DA4` `dword_71A000` (the
//! setup's gearing, a whole number) over `flt_5C5D00` and `flt_71A1B4` (hi / lo
//! `flt_5CB978` / `flt_5CB970` and `flt_5CB974` / `flt_5CB96C`); the gears between in
//! even steps ((top - first) / (count - 1), kept at 80 bits on the stack and added on:
//! gears 2 .. 5 always, 6 and 7 one more step each only past five and six gears; the
//! step itself rounded into `dword_62400C`); reverse (`flt_6140BC`) the first's negative;
//! neutral (`g_gearRatios[0]`) `flt_613FE8`.
//!
//! In:  nothing
//! Out: ebx `g_gearCount`; the rest kept; no flags read
//!
//! Callers: `sub_43B635` (0x43B63F) and `sub_4217E4` (the car specification's).
//! Refuses (before anything) a gear count above 7 (past `g_gearRatios`).
//! Self-test: `physics/copy_hooks_test.rs` (`test_moves_floats`: a copy on test
//! variables against the converted version), counts 4 .. 7 and setups by lot, exact at
//! double precision; the refusal.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, rva};
use crate::math::x87;
use crate::math::x87_run::{ext_add, gear_step_80, lerp_int_over2};
use crate::scene::matrix3::{extended, rounded};

/// The count, the four setup ends, the setup's gearing, the divisor, then the ratios from
/// reverse (0x6140BC) to gear 7 (0x6140DC), the scratch, neutral's.
const VARIABLES: [usize; 18] = [
    0x614C10, 0x5CB978, 0x5CB970, 0x5CB974, 0x5CB96C, 0x71A000, 0x71A1B4, 0x6140BC, 0x6140C0,
    0x6140C4, 0x6140C8, 0x6140CC, 0x6140D0, 0x6140D4, 0x6140D8, 0x6140DC, 0x62400C, 0x613FE8,
];
const RATIO: usize = 8; // index of gear 0 in `VARIABLES`

/// The Rust core on the variables at `v`: the gear count.
pub unsafe fn ratios_at(v: &[u32]) -> Result<u32, Refused> {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        let n = get(0);
        if n > 7 {
            return Err(BadInput("a gear count above 7 (past g_gearRatios)").into());
        }
        let (k, d) = (
            game_read::<u32>(rva(0x5C5DA4)),
            game_read::<u32>(rva(0x5C5D00)),
        );
        let one: u32 = game_read(rva(0x5C5D38));
        let i = get(5) as i32;
        put(
            RATIO + n as usize,
            lerp_int_over2(get(1), get(2), k, i, d, get(6)),
        );
        put(RATIO + 1, lerp_int_over2(get(3), get(4), k, i, d, get(6)));
        let step = gear_step_80(get(RATIO + n as usize), get(RATIO + 1), n as i32, one);
        let mut acc = extended(get(RATIO + 1));
        for gear in 2..=5 {
            acc = ext_add(acc, step);
            put(RATIO + gear, rounded(acc));
        }
        if n > 5 {
            acc = ext_add(acc, step);
        }
        put(RATIO + 6, rounded(acc));
        if n > 6 {
            acc = ext_add(acc, step);
        }
        put(RATIO + 7, rounded(acc));
        put(16, rounded(step));
        put(7, x87::negated(get(RATIO + 1)));
        put(RATIO, x87::loaded(get(17)));
        Ok(n)
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::test_drive`): the gear count.
pub unsafe fn make_gear_ratios() -> Result<u32, Refused> {
    unsafe { ratios_at(&game()) }
}
