//! `physics::camera_clear_of_ground` (0x438DD2): whether the camera is a whole unit
//! (0x10000) or more above the ground under it (`unk_71A680` - `dword_639828`, by the
//! sign of the `cmp`; `physics/camera_ground.rs`): CF clear when found and so high, else
//! CF.
//!
//! In:  nothing
//! Out: CF; eax as the steps leave it, the rest as `count_ground_probe` leaves them (the
//!      caller reads CF)
//!
//! Caller: `sub_43907F` (three).
//! Refuses what `count_ground_probe` refuses. Shadow mode runs the original alone.
//! Self-test: a copy whose probe goes to a stand-in, against the Rust version with the
//! same: found or not, heights about the limit.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::camera_ground::{carry_entry, probe_at};
use crate::physics::flow::g;
use crate::race::car_starts::Part;
use rc2k_core::physics::camera_ground::{is_clear, over};

/// The Rust version, the probe through `part`; CF in the registers left.
// Converted: `rc2k_core::physics::camera_ground::{over, is_clear}`.
pub unsafe fn camera_clear_of_ground(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = g(0x71A67C);
    let x = r.eax;
    r.eax = g(0x71A684);
    let z = r.eax;
    let mut clear = probe_at(&mut r, part, x, z, 0x71AEC8)?;
    if clear {
        r.eax = over(g(0x71A680), g(0x639828));
        clear = is_clear(r.eax);
    }
    r.set_carry(!clear);
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    carry_entry(r, camera_clear_of_ground)
}
