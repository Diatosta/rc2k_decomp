//! `physics::reset_car_on_line` (0x43FB00): the work car put back on the line at the
//! checkpoint nearest it: `race::nearest_checkpoint` into `dword_71A070`, `6C`, `74`
//! (`g_car` +0x98, +0x94, +0x9C), `dword_6365E4` and `dword_6365EC`;
//! `race::place_car_on_line`, `race::reset_car_motion`; then +0xFB0 and +0xFB4
//! (`dword_71AF88`, `8C`) 8 and +0xFAC (`dword_71AF84`) 10.
//!
//! In:  nothing (the registers are passed on to the calls)
//! Out: as `reset_car_motion` leaves them; no flags read (the caller's next is a `mov`)
//!
//! It begins with a call: hooked by a private copy with its three calls relocated and a
//! hand-made patch.
//! Caller: `physics::reset_car` (its end).
//! Refuses what the three calls refuse. Shadow mode runs the original alone
//! (`physics/reset_parts.rs`).
//! Self-test: a copy whose calls go to the stand-ins, against the Rust version with the
//! same, on stream A's cases; the work car and the start's variables put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_write, rva};
use crate::physics::reset_parts::reset_entry;

/// The Rust version, the calls through `part` (places 8, 6, 9).
pub unsafe fn reset_car_on_line(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        crate::physics::reset_parts::game_part(8, &mut g)?;
        for v in [0x71A070usize, 0x71A06C, 0x71A074, 0x6365E4, 0x6365EC] {
            game_write(rva(v), g.eax);
        }
        crate::physics::reset_parts::game_part(6, &mut g)?;
        crate::physics::reset_parts::game_part(9, &mut g)?;
        game_write(rva(0x71AF88), 8u32);
        game_write(rva(0x71AF8C), 8u32);
        game_write(rva(0x71AF84), 10u32);
    }
    Ok(g)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    reset_entry(r, reset_car_on_line)
}
