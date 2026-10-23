//! `race::start_car` (0x4464AD): the work car set at its start: its checkpoint on the
//! best line (by its number), its place there, back and to the side, the nearest
//! checkpoint and its check count, then `race::reset_car_motion`.
//!
//! In:  nothing (the registers are passed on to the calls)
//! Out: the registers as `reset_car_motion` leaves them
//!
//! See `race/car_starts.rs`. Callers: `race::start_cars` (race init) and `sub_43B635`
//! (0x43BA88, stream B's). Refuses a nearest checkpoint past the lists (when met; the
//! original reads past `g_checkCounts`). Shadow mode runs the original alone
//! (`settle_car`'s game calls). Self-test: a copy whose calls go to stand-ins, against
//! the Rust version on the same: every branch of `dword_62389C` (0 .. 4), `dword_64BB18`,
//! `dword_52A910`, `dword_63DB94`, odd and even cars.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::car_starts::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { start_car(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
