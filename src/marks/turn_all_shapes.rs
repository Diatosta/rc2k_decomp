//! `marks::turn_all_shapes` (0x4BD122): for each of the settings' objects the shape set
//! turned by the two shapes' eased steps.
//!
//! In:  nothing
//! Out: esi the last object's offset (the settings with none), the rest as the last turn
//!
//! See `marks/shape_points.rs`. One caller, 0x4BDF87. The object's offset is not used: the
//! same set is turned each time (section 16). Refuses what `marks::turn_shape_points` would,
//! before writing. Self-test: the original and the Rust version with 0, 1 and 3 objects,
//! steps, switches and sets by lot, the refusal; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::shape_points::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, state_entry};

pub fn regions() -> Vec<Region> {
    let mut r = turn_regions();
    r.extend(unsafe { target_regions() });
    r.extend(crate::marks::turns::step_regions());
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { turn_all_shapes(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
