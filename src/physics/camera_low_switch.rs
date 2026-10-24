//! `physics::camera_low_switch` (0x438BF4): with `unk_71A0A0` below 0 (or unordered), no
//! switch under way (`unk_71A66C` 0) and the camera in mode 0 or 1 (`unk_71A65C`): the
//! ground under the camera (`unk_71A67C`, `unk_71A684`; `physics/camera_ground.rs`);
//! found and the camera less than 0x800 above it (`unk_71A680` - `dword_639828`, by the
//! sign of the `cmp`): the switch begun: `unk_71A664` = the mode before (`unk_71A660`)
//! plus 1, `unk_71A660` = 4, `unk_71A66C` = 1, `unk_71A674` = 0, `unk_71A670` = 10,
//! `unk_71A668` = the clock (`unk_719FE0`).
//!
//! In:  nothing
//! Out: eax as the steps leave it, the rest as `count_ground_probe` leaves them; no flags
//!      read (the caller goes on)
//!
//! Caller: `sub_437A53`.
//! Refuses what `count_ground_probe` refuses. Shadow mode runs the original alone.
//! Self-test: a copy whose probe goes to a stand-in, against the Rust version with the
//! same: every early way out, found or not, low or not.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::camera_ground::probe_at;
use crate::physics::flow::{c0, g, p};
use rc2k_core::physics::camera_ground::{is_low, low_switch, low_switch_looks, over};

/// The Rust version. The eax it leaves is read by none of its callers.
// Converted: `rc2k_core::physics::camera_ground::{low_switch_looks, is_low, low_switch}`.
pub fn camera_low_switch() -> Result<(), Refused> {
    if !low_switch_looks(c0(g(0x71A0A0), g(0x5C5D90)), g(0x71A66C), g(0x71A65C)) {
        return Ok(());
    }
    if !probe_at(g(0x71A67C), g(0x71A684), 0x71AEC8)? {
        return Ok(());
    }
    if !is_low(over(g(0x71A680), g(0x639828))) {
        return Ok(());
    }
    let s = low_switch(g(0x71A660), g(0x719FE0));
    p(0x71A664, s.before);
    p(0x71A660, s.mode);
    p(0x71A66C, s.switching);
    p(0x71A674, s.w674);
    p(0x71A670, s.w670);
    p(0x71A668, s.clock);
    Ok(())
}
