//! `race::set_up_mode_0_cars` (0x4364BD): `sub_43603C`'s set-up for `dword_62389C` 0:
//! `race::reset_car_blocks`; with `dword_52A910` 0 the two-player rule, the players'
//! kinds, both players' automatic set-ups and their +0x3C from `dword_75AD2C` /
//! `dword_75AE98` (below 0: 0, 100 or more: 0x100, else x 2.56 rounded); else every
//! race car's automatic set-up, then the players' kinds.
//!
//! In:  nothing
//! Out: the registers as the steps leave them (esi the second player's block; with
//!      `dword_52A910`, ecx the count)
//!
//! See `race/race_cars.rs`. One caller, `sub_43603C`. It begins with its call: private
//! copy, its seven calls relocated, manual patch. Refuses, with `dword_52A910` set, no
//! cars or more than 8 (the original's `jnz` would run on), before anything. Shadow mode
//! runs the original alone (the set-ups log). Self-test: a copy whose calls note
//! themselves, on the game's blocks (filled, put back), both paths, 1 .. 8 cars, the
//! values round 0 and 100; the refusal on the Rust version alone.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::race_cars::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { set_up_mode_0_cars(r) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
