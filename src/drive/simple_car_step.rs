//! The free-roaming car's step (no physics, the race mode
//! `levers_on` 0). The inputs (`drive::car_input_words`, `drive::read_car_controls`,
//! `drive::steer_inputs`, `drive::pedal_inputs`, `drive::reset_request`); the car moved along its
//! heading (`car_heading`) by (`throttle` - `brake`) << 5, turned by `steering` >> 4; the wheels' surfaces; the
//! ground under `place_x`, `place_z` (`physics::find_ground`), found: the height
//! `ground_height` + `ground_drop`'s bits into `place_y` and the height word of `position`; pitch and roll 0;
//! `race::get_mat_crash`. The first car block throughout.
//!
//! Refuses what its calls refuse.

use crate::errors::Refused;
use crate::physics::figures::GROUND_DROP;
use crate::race::owner::Race;
use rc2k_core::drive::roaming::{roam_distance, roam_turn};

pub fn simple_car_step(
    race: &Race,
    input: &crate::input::owner::Input,
    scene: &crate::scene::owner::Scene,
    net: &crate::net::owner::Net,
) -> Result<(), Refused> {
    crate::drive::car_input_words::car_input_words(race, input, net)?;
    crate::drive::read_car_controls::read_car_controls(race, scene, net)?;
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
    let position = &race.car_blocks().cars[0].position;
    position.set_bits_at(0, position.bits_at(0).wrapping_add(x));
    position.set_bits_at(2, position.bits_at(2).wrapping_add(z));
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
    {
        let mut search = race.physics.ground_search().borrow_mut();
        let place = *race.course.course.borrow();
        search.x = place.place_x;
        search.z = place.place_z;
    }
    {
        crate::physics::find_ground::find_ground(
            race.physics.ground_probe(),
            race.physics.ground_search(),
            &race.course,
        )
    }?;
    if race.physics.ground_search().borrow().slot != u32::MAX {
        let y = race
            .physics
            .ground_search()
            .borrow()
            .height
            .wrapping_add(GROUND_DROP);
        race.course.course.borrow_mut().place_y = y;
        race.car_blocks().cars[0].position.set_bits_at(1, y);
    }
    race.car_blocks().cars[0].car_pitch.set(0);
    race.car_blocks().cars[0].car_roll.set(0);
    crate::race::stage_prefs::get_mat_crash(
        race.engine_tables(),
        &race.frame.matrices,
        race.shared.car(),
        scene,
    );
    Ok(())
}
