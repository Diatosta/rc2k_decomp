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
//! Caller: `sub_437A53`. Refuses what `count_ground_probe` refuses.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::camera_ground::probe_at;
use crate::physics::flow::c0;
use crate::race::owner::Race;
use rc2k_core::physics::camera_ground::{is_low, low_switch, low_switch_looks, over};

/// The Rust version. The eax it leaves is read by none of its callers.
// Converted: `rc2k_core::physics::camera_ground::{low_switch_looks, is_low, low_switch}`.
pub fn camera_low_switch(race: &Race) -> Result<(), Refused> {
    if !low_switch_looks(
        c0(
            crate::race::owner::race().shared.car.crash_matrix[5].bits(),
            race.course.numbers.zero.bits(),
        ),
        crate::race::owner::race()
            .shared
            .car
            .camera_switch_active
            .get(),
        crate::race::owner::race().shared.car.camera_view.get(),
    ) {
        return Ok(());
    }
    if !probe_at(
        race,
        crate::race::owner::race().shared.car.camera_position[0].get(),
        crate::race::owner::race().shared.car.camera_position[2].get(),
        &crate::race::owner::race().shared.car.camera_ground_faces[0],
    )? {
        return Ok(());
    }
    if !is_low(over(
        crate::race::owner::race().shared.car.camera_position[1].get(),
        race.physics.ground.ground_height.get(),
    )) {
        return Ok(());
    }
    let s = low_switch(
        crate::race::owner::race()
            .shared
            .car
            .camera_blend_view
            .get(),
        crate::race::owner::race().shared.car.step_clock.get(),
    );
    crate::race::owner::race()
        .shared
        .car
        .camera_view_return
        .set(s.before);
    crate::race::owner::race()
        .shared
        .car
        .camera_blend_view
        .set(s.mode);
    crate::race::owner::race()
        .shared
        .car
        .camera_switch_active
        .set(s.switching);
    crate::race::owner::race()
        .shared
        .car
        .camera_switch_share
        .set_bits(s.w674);
    crate::race::owner::race()
        .shared
        .car
        .camera_switch_length
        .set(s.w670);
    crate::race::owner::race()
        .shared
        .car
        .camera_switch_clock
        .set(s.clock);
    Ok(())
}
