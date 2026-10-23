//! `race::set_up_car_points` (0x426B67): a car's collision points into its block.
//!
//! In:  esi = the car's slot, edi = its block
//! Out: eax the last moved z, ebx the slot's first point + the count, ecx 3, edx the
//!      count (with no points ecx 0); the rest preserved
//!
//! See `race/objects.rs`. One caller, `race::set_object_kinds` (inside `pusha`). Refuses
//! more than 0x40 points, and one (the original returns through its saved edi). Shadow mode runs the original alone (it logs). Self-test:
//! a copy of the original whose five variables are the test's and whose log calls go to
//! stand-ins (the sorts stay on their hooks), so that it runs paused and the frames drawn
//! meanwhile are not touched (the game's `g_objectPoints` changed for the test made a menu
//! object's geometry fly apart, 2026-10-16); points and a block of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::objects::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { set_up_car_points(r, &CarPointVars::game(), &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
