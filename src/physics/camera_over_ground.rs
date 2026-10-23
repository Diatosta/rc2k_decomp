//! `physics::camera_over_ground` (0x438CEC): whether there is ground under the camera
//! and a half-unit (0x8000) to either side across its heading (`unk_71A688`,
//! `math::turn_point_xz` of (-0x8000, y, 0) and (0x8000, y, 0)), each probe from its own
//! last face (`unk_71AEC8`, `unk_71AECC`, `unk_71AED0`; `physics/camera_ground.rs`).
//! CF clear when all three are found; the first not found sets CF and ends it.
//!
//! In:  nothing (ebx passed to the turns as y)
//! Out: CF; eax, ecx, edx as the turns and probes leave them, the rest as
//!      `count_ground_probe` leaves them (the caller reads CF)
//!
//! Caller: `sub_43907F` (four).
//! Refuses what `count_ground_probe` refuses. Shadow mode runs the original alone.
//! Self-test: a copy whose probes go to a stand-in, against the Rust version with the
//! same: found and not, headings by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::math::turn_point_xz::turn_point_xz;
use crate::physics::camera_ground::{carry_entry, probe_at};
use crate::physics::flow::{g, p};
use crate::race::car_starts::Part;
use rc2k_core::physics::camera_ground::SIDES;

// Converted: `rc2k_core::physics::camera_ground::SIDES`.
fn steps(r: &mut Regs, part: Part) -> Result<bool, Refused> {
    r.eax = g(0x71A67C);
    let x = r.eax;
    r.eax = g(0x71A684);
    let z = r.eax;
    if !probe_at(r, part, x, z, 0x71AEC8)? {
        return Ok(false);
    }
    for (side, slot) in SIDES.into_iter().zip([0x71AECC, 0x71AED0]) {
        p(0x5CCC78, g(slot));
        let [x, _, z, edx] = unsafe { turn_point_xz(side as i32, r.ebx as i32, 0, g(0x71A688)) };
        r.eax = x.wrapping_add(g(0x71A67C));
        r.ecx = z.wrapping_add(g(0x71A684));
        r.edx = edx;
        let (x, z) = (r.eax, r.ecx);
        if !probe_at(r, part, x, z, slot)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The Rust version, the probes through `part`; CF in the registers left.
pub unsafe fn camera_over_ground(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let all = steps(&mut r, part)?;
    r.set_carry(!all);
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    carry_entry(r, camera_over_ground)
}
