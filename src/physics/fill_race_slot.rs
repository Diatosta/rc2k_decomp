//! Puts a loose part (its number) into one of the eight race slots (`race_slots`, 0x78 bytes each):
//! the first empty one, else the one with the lowest owner number (unsigned, the first of equals;
//! when every one is -1 it is refused). The slot gets the part; its middle (the part's
//! `part_extents`, whole numbers) turned by the crash matrix, rounded and added to the car's place;
//! the car's `ground_bump_scale` and number; the car's matrix (16 dwords); the turned middle times
//! `part_fling` plus the car's motion (+2.0 for y) as its speed; and the car's spin. A reading: a
//! part breaking off, flying with the car's motion.
//!
//! Refuses (before anything) a part past 0xFF, or no slot to take.

use crate::errors::{BadInput, Refused};
use crate::math::x87::{ext_to_int, int_80, int_mul_add};
use crate::physics::car_store::Car;
use crate::race::numbers_store::Numbers;
use crate::race::work_store::RaceWork;
use crate::scene::rotate_point::rotate_point;
use rc2k_core::physics::parts::{LAST_PART, choose_slot};

/// Part `part` into a race slot.
// Core: `rc2k_core::physics::parts::choose_slot` (the slot; the floats stay x87).
pub fn fill_race_slot(
    numbers: &Numbers,
    work: &mut RaceWork,
    car: &Car,
    part: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    if part > LAST_PART {
        return Err(BadInput("a part past 0xFF").into());
    }
    let Some(choice) = choose_slot(|n| work.race_slots[n].part, |n| work.race_slots[n].age) else {
        return Err(BadInput("no race slot to take (every +0x64 -1)").into());
    };
    let k = work.part_fling;
    let slot = &mut work.race_slots[choice.slot];
    slot.part = part;
    let i = |t: usize| {
        int_80(
            scene.loaded_slots().part_extents[t][part as usize] as i32,
            false,
        )
    };
    let out = rotate_point(&car.crash_matrix, [i(0), i(1), i(2)]);
    for (k, e) in out.iter().enumerate() {
        let n = ext_to_int(*e) as u32;
        scene.object_work_mut().object_position[k] = n;
        slot.place[k] = n.wrapping_add(car.position.bits_at(k));
    }
    slot.car_word = car.probe_point_face[4].get();
    slot.age = car.step_clock.get();
    slot.matrix = car.crash_matrix.get();
    slot.motion[0] = int_mul_add(
        scene.object_work().object_position[0] as i32,
        k,
        car.velocity.bits_at(0),
        None,
    );
    slot.motion[1] = int_mul_add(
        scene.object_work().object_position[1] as i32,
        k,
        car.velocity.bits_at(1),
        Some(numbers.two.bits()),
    );
    slot.motion[2] = int_mul_add(
        scene.object_work().object_position[2] as i32,
        k,
        car.velocity.bits_at(2),
        None,
    );
    for (k, spin) in slot.spin.iter_mut().enumerate() {
        *spin = car.spin.at(k);
    }
    Ok(())
}
