//! `drive::simple_car_step` (0x43A1CA): the free-roaming car's step (no physics, the race
//! mode `dword_621884` 0). The inputs (`drive::car_input_words`, `drive::read_car_controls`,
//! `drive::steer_inputs`, `drive::pedal_inputs`, `drive::reset_request`); the car moved
//! along its heading (+0x9C0) by (+0x638 - +0x63C) << 5 (<< 7 with `byte_4F6B79`,
//! `math::turn_point_xz` of (0, ebx, it)), turned by +0x644 >> 4; the wheels' surfaces;
//! the ground under `dword_5CCDA0`, `5CCDE0` (`physics::find_ground`), found: the height
//! `dword_639828` + `flt_613FC4`'s bits into `dword_5CCDC0` and +0x138; pitch and roll 0;
//! `race::get_mat_crash`. The first car block (`g_carBlockStore`) throughout.
//!
//! In:  nothing
//! Out: the registers as the steps and calls leave them; no flags read
//!
//! Caller: `drive::race_tick`. Refuses what its calls refuse.

use crate::errors::Refused;
use crate::input::owner::input;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::Race;
use rc2k_core::drive::roaming::{roam_distance, roam_turn};

/// The Rust version. The registers it leaves are read by no caller (liveness: ecx, edi
/// and esi after `drive::race_tick`'s call have no read point).
// Converted: `rc2k_core::drive::roaming::{roam_distance, roam_turn}`.
pub fn simple_car_step(race: &Race) -> Result<(), Refused> {
    crate::drive::car_input_words::car_input_words()?;
    crate::drive::read_car_controls::read_car_controls()?;
    crate::drive::steer_inputs::steer_inputs();
    crate::drive::pedal_inputs::pedal_inputs();
    // The keys `drive::pedal_inputs` leaves in ebx (`reset_request` loads the same again
    // before `physics::reset_car`): the turn's y.
    let keys = race.shared.car.input_keys.get();
    { crate::drive::reset_request::reset_request() }?;
    let distance = roam_distance(
        CAR_BLOCKS.cars[0].throttle.bits(),
        CAR_BLOCKS.cars[0].brake.bits(),
        input().devices.keys_now.get(0x1D) != 0, // byte_4F6B79
    );
    let [x, _, z, _] = {
        crate::math::turn_point_xz::turn_point_xz(
            0,
            keys as i32,
            distance as i32,
            CAR_BLOCKS.cars[0].car_heading.bits(),
        )
    };
    CAR_BLOCKS.cars[0].position[0].set_bits(CAR_BLOCKS.cars[0].position[0].bits().wrapping_add(x));
    CAR_BLOCKS.cars[0].position[2].set_bits(CAR_BLOCKS.cars[0].position[2].bits().wrapping_add(z));
    CAR_BLOCKS.cars[0].car_heading.set_bits(
        CAR_BLOCKS.cars[0]
            .car_heading
            .bits()
            .wrapping_add(roam_turn(CAR_BLOCKS.cars[0].steering.bits())),
    );
    { crate::physics::wheel_surfaces::wheel_surfaces(race) }?;
    race.physics
        .ground
        .ground_x
        .set(race.course.course.place_x.get());
    race.physics
        .ground
        .ground_z
        .set(race.course.course.place_z.get());
    { crate::physics::find_ground::find_ground(race) }?;
    if race.course.course.ground_slot.get() != u32::MAX {
        let y = race
            .physics
            .ground
            .ground_height
            .get()
            .wrapping_add(race.shared.physics_tables.ground_drop.get());
        race.course.course.place_y.set(y);
        CAR_BLOCKS.cars[0].position[1].set_bits(y);
    }
    CAR_BLOCKS.cars[0].car_pitch.set_bits(0);
    CAR_BLOCKS.cars[0].car_roll.set_bits(0);
    // `race::get_mat_crash` reads no register at entry (liveness): a scratch.
    crate::race::stage_prefs::get_mat_crash();
    Ok(())
}
