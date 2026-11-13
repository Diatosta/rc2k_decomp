//! The free-roaming car's step (no physics, the race mode
//! `levers_on` 0). The inputs (`drive::car_input_words`, `drive::read_car_controls`,
//! `drive::steer_inputs`, `drive::pedal_inputs`, `drive::reset_request`); the car moved along its
//! heading (+0x9C0) by (+0x638 - +0x63C) << 5, turned by +0x644 >> 4; the wheels' surfaces; the
//! ground under `place_x`, `place_z` (`physics::find_ground`), found: the height
//! `ground_height` + `ground_drop`'s bits into `place_y` and +0x138; pitch and roll 0;
//! `race::get_mat_crash`. The first car block throughout.
//!
//! Refuses what its calls refuse.

use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::drive::roaming::{roam_distance, roam_turn};

pub fn simple_car_step(
    race: &Race,
    input: &crate::input::owner::Input,
    scene: &crate::scene::owner::Scene,
    net: &crate::net::owner::Net,
) -> Result<(), Refused> {
    crate::drive::car_input_words::car_input_words(race, input, net)?;
    crate::drive::read_car_controls::read_car_controls(race, net, scene)?;
    crate::drive::steer_inputs::steer_inputs(race, scene);
    crate::drive::pedal_inputs::pedal_inputs(race);
    // The keys `drive::pedal_inputs` leaves (`reset_request` loads the same again
    // before `physics::reset_car`): the turn's y.
    let keys = race.shared.car().input_keys.get();
    { crate::drive::reset_request::reset_request(race, scene) }?;
    let distance = roam_distance(
        race.car_blocks().cars[0].throttle.get(),
        race.car_blocks().cars[0].brake.get(),
        input.devices.keys_now.get(0x1D) != 0, // Devices.keys_now
    );
    let [x, _, z, _] = {
        crate::math::rotate_point_about_y::rotate_point_about_y(
            race.engine_tables(),
            0,
            keys as i32,
            distance as i32,
            race.car_blocks().cars[0].car_heading.get(),
        )
    };
    race.car_blocks().cars[0].position[0]
        .set(race.car_blocks().cars[0].position[0].get().wrapping_add(x));
    race.car_blocks().cars[0].position[2]
        .set(race.car_blocks().cars[0].position[2].get().wrapping_add(z));
    race.car_blocks().cars[0].car_heading.set(
        race.car_blocks().cars[0]
            .car_heading
            .get()
            .wrapping_add(roam_turn(race.car_blocks().cars[0].steering.get())),
    );
    {
        crate::physics::wheel_surfaces::wheel_surfaces(
            &crate::physics::step_inputs::StepInputs::of(race).wheels(),
            scene,
        )
    }?;
    race.physics
        .ground()
        .ground_x
        .set(race.course.course.get().place_x.get());
    race.physics
        .ground()
        .ground_z
        .set(race.course.course.get().place_z.get());
    {
        crate::physics::find_ground::find_ground(
            race.physics.ground_probe.get(),
            race.physics.ground(),
            &race.course,
            scene,
        )
    }?;
    if race.course.course.get().ground_slot.get() != u32::MAX {
        let y = race
            .physics
            .ground()
            .ground_height
            .get()
            .wrapping_add(race.shared.physics_tables.get().ground_drop.get());
        race.course.course.get().place_y.set(y);
        race.car_blocks().cars[0].position[1].set(y);
    }
    race.car_blocks().cars[0].car_pitch.set(0);
    race.car_blocks().cars[0].car_roll.set(0);
    crate::race::stage_prefs::get_mat_crash(
        race.engine_tables(),
        race.frame.matrices.get(),
        race.shared.car(),
        scene,
    );
    Ok(())
}
