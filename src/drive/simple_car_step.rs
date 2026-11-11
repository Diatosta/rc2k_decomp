//! `drive::simple_car_step` (0x43A1CA): the free-roaming car's step (no physics, the race mode
//! `levers_on` 0). The inputs (`drive::car_input_words`, `drive::read_car_controls`,
//! `drive::steer_inputs`, `drive::pedal_inputs`, `drive::reset_request`); the car moved along its
//! heading (+0x9C0) by (+0x638 - +0x63C) << 5, turned by +0x644 >> 4; the wheels' surfaces; the
//! ground under `place_x`, `5CCDE0` (`physics::find_ground`), found: the height
//! `ground_height` + `ground_drop`'s bits into `place_y` and +0x138; pitch and roll 0;
//! `race::get_mat_crash`. The first car block (`g_carBlockStore`) throughout.
//!
//! Caller: `drive::race_tick`. Refuses what its calls refuse.

use crate::errors::Refused;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::Race;
use rc2k_core::drive::roaming::{roam_distance, roam_turn};

// Converted: `rc2k_core::drive::roaming::{roam_distance, roam_turn}`.
pub fn simple_car_step(
    race: &Race,
    input: &crate::input::owner::Input,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    crate::drive::car_input_words::car_input_words(race, input)?;
    crate::drive::read_car_controls::read_car_controls(race)?;
    crate::drive::steer_inputs::steer_inputs(race);
    crate::drive::pedal_inputs::pedal_inputs(race);
    // The keys `drive::pedal_inputs` leaves (`reset_request` loads the same again
    // before `physics::reset_car`): the turn's y.
    let keys = race.shared.car.input_keys.get();
    { crate::drive::reset_request::reset_request(race, scene) }?;
    let distance = roam_distance(
        CAR_BLOCKS.cars[0].throttle.get(),
        CAR_BLOCKS.cars[0].brake.get(),
        input.devices.keys_now.get(0x1D) != 0, // Devices.keys_now
    );
    let [x, _, z, _] = {
        crate::math::rotate_point_about_y::rotate_point_about_y(
            0,
            keys as i32,
            distance as i32,
            CAR_BLOCKS.cars[0].car_heading.get(),
        )
    };
    CAR_BLOCKS.cars[0].position[0].set(CAR_BLOCKS.cars[0].position[0].get().wrapping_add(x));
    CAR_BLOCKS.cars[0].position[2].set(CAR_BLOCKS.cars[0].position[2].get().wrapping_add(z));
    CAR_BLOCKS.cars[0].car_heading.set(
        CAR_BLOCKS.cars[0]
            .car_heading
            .get()
            .wrapping_add(roam_turn(CAR_BLOCKS.cars[0].steering.get())),
    );
    {
        crate::physics::wheel_surfaces::wheel_surfaces(
            &crate::physics::step_inputs::StepInputs::of(race).wheels(),
            scene,
        )
    }?;
    race.physics
        .ground
        .ground_x
        .set(race.course.course.place_x.get());
    race.physics
        .ground
        .ground_z
        .set(race.course.course.place_z.get());
    {
        crate::physics::find_ground::find_ground(
            &race.physics.ground_probe,
            &race.physics.ground,
            &race.course,
            scene,
        )
    }?;
    if race.course.course.ground_slot.get() != u32::MAX {
        let y = race
            .physics
            .ground
            .ground_height
            .get()
            .wrapping_add(race.shared.physics_tables.ground_drop.get());
        race.course.course.place_y.set(y);
        CAR_BLOCKS.cars[0].position[1].set(y);
    }
    CAR_BLOCKS.cars[0].car_pitch.set(0);
    CAR_BLOCKS.cars[0].car_roll.set(0);
    crate::race::stage_prefs::get_mat_crash(&race.frame.matrices, &race.shared.car);
    Ok(())
}
