//! `race::cast_car_shadows` (0x45754C): every race car's shadow (see `race/car_shadow.rs`):
//! the depth test off; for each car with +0x6FC set, loaded into the work copy, its corner
//! count from `g_car` +0x260, `race::cast_car_shadow` once with `dword_6238B0` and once more
//! with `dword_60933C` 0x40; the depth test on.
//!
//! In:  nothing
//! Out: ecx the cars, esi the last block, eax 1, ebx 0xE (the depth test's call; a `call`
//!      follows)
//!
//! Caller: `sub_4557AB` (0x45584E), each view drawn. Refuses (before anything) no cars or more
//! than eight, or a null block. Hook removed 2026-10-22 (direct calls): Rust callers run its
//! `entry` (`calls::call`, `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::race::car_shadow::*;
use crate::race::cast_car_shadow::faces_in_game;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let done = unsafe {
        cast_car_shadows(r, &mut depth_part, &mut |e| {
            cast_car_shadow(e, &mut faces_in_game).map(|_| ())
        })
    };
    {
        let after = done?;
        let esp = r.esp;
        *r = after;
        r.esp = esp;
    }
    Ok(())
}
