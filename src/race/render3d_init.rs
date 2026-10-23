//! `race::render3d_init` (0x4433A1): `scene::allocate_point_records` (`render3d init`; a `call` and a `retn`).
//!
//! In:  nothing
//! Out: as `scene::allocate_point_records` leaves them
//!
//! See `race/level_init.rs`. One caller, `sub_44426F` (`race::load_stage`'s
//! "loadlevel_init"). Shadow mode runs the original alone. Self-test: a redirected copy
//! whose variables and memory are the test's and whose calls note themselves.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::level_init::*;

fn rust(r: &Regs, _: &Vars) -> Result<Regs, crate::errors::Refused> {
    let mut g = *r;
    game_part(2, &mut g)?;
    Ok(g)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = rust(r, &Vars::game())?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}
