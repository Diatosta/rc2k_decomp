//! `physics::camera_low_switch` (0x438BF4): with `unk_71A0A0` below 0 (or unordered), no
//! switch under way (`camera_switch_active` 0) and the camera in mode 0 or 1 (`camera_view`): the
//! ground under the camera (`camera_position`, `unk_71A684`; `physics/camera_ground.rs`);
//! found and the camera less than 0x800 above it (`unk_71A680` - `ground_height`, by the
//! sign of the `cmp`): the switch begun: `camera_view_return` = the mode before (`camera_blend_view`)
//! plus 1, `camera_blend_view` = 4, `camera_switch_active` = 1, `camera_switch_share` = 0, `camera_switch_length` = 10,
//! `camera_switch_clock` = the clock (`step_clock`).
//!
//! Caller: `physics::update_camera`. Refuses what `count_ground_probe` refuses.

use crate::errors::Refused;
use crate::physics::camera_ground::probe_at;
use crate::physics::flow::{ZERO_BITS as ZERO, c0};
use crate::race::owner::Race;
use rc2k_core::physics::camera_ground::{is_low, low_switch, low_switch_looks, over};

// Converted: `rc2k_core::physics::camera_ground::{low_switch_looks, is_low, low_switch}`.
pub fn camera_low_switch(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    if !low_switch_looks(
        c0(race.shared.car.crash_matrix[5].bits(), ZERO),
        race.shared.car.camera_switch_active.get(),
        race.shared.car.camera_view.get(),
    ) {
        return Ok(());
    }
    if !probe_at(
        race,
        race.shared.car.camera_position[0].get(),
        race.shared.car.camera_position[2].get(),
        &race.shared.car.camera_ground_faces[0],
        scene,
    )? {
        return Ok(());
    }
    if !is_low(over(
        race.shared.car.camera_position[1].get(),
        race.physics.ground.ground_height.get(),
    )) {
        return Ok(());
    }
    let s = low_switch(
        race.shared.car.camera_blend_view.get(),
        race.shared.car.step_clock.get(),
    );
    race.shared.car.camera_view_return.set(s.before);
    race.shared.car.camera_blend_view.set(s.mode);
    race.shared.car.camera_switch_active.set(s.switching);
    race.shared.car.camera_switch_share.set_bits(s.w674);
    race.shared.car.camera_switch_length.set(s.w670);
    race.shared.car.camera_switch_clock.set(s.clock);
    Ok(())
}
