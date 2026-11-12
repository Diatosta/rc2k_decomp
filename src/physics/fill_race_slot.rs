//! `physics::fill_race_slot` (0x462263): a loose part (its number) put into one of the eight
//! `g_raceSlots` (0x78 bytes each): the first empty one (+0 is 0), else the one with the lowest
//! +0x64 (unsigned, the first of equals; with every +0x64 -1 the original uses an uninitialised
//! slot pointer: refused). There: +0 the part; its middle (`g_partMiddleX` / `Y` / `Z`[part],
//! whole numbers) turned by `g_crashMatrix` (`scene::rotate_point`), rounded and added to the car's
//! place into +4 .. +0xC; +0x60 the car's `ground_bump_scale`, +0x64 the car's number; the car's
//! matrix (16 dwords) at +0x10; +0x50 .. +0x58 the turned middle times `part_fling` plus the car's
//! motion (+2.0 for y); +0x68 .. +0x70 the car's spin. A reading: a part breaking off, flying with
//! the car's motion.
//!
//! Takes: the part
//!
//! Caller: `physics::fill_slot_on_bit`. Refuses (before anything) a part past 0xFF, or no
//! slot to take.

use crate::errors::{BadInput, Refused};
use crate::math::x87::{ext_to_int, int_80, int_mul_add};
use crate::physics::car_store::Car;
use crate::race::numbers_store::Numbers;
use crate::race::work_store::RaceWork;
use crate::scene::rotate_point::rotate_point;
use rc2k_core::physics::parts::{LAST_PART, choose_slot};

/// Part `part` into a race slot.
// Converted: `rc2k_core::physics::parts::choose_slot` (the slot; the floats stay x87).
pub fn fill_race_slot(
    numbers: &Numbers,
    work: &RaceWork,
    car: &Car,
    part: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    if part > LAST_PART {
        return Err(BadInput("a part past 0xFF").into());
    }
    let Some(choice) = choose_slot(
        |n| work.race_slots[n].part.get(),
        |n| work.race_slots[n].age.get(),
    ) else {
        return Err(BadInput("no race slot to take (every +0x64 -1)").into());
    };
    let slot = &work.race_slots[choice.slot];
    slot.part.set(part);
    let i = |t: usize| {
        int_80(
            scene.objects.slots.part_extents[t][part as usize].get() as i32,
            false,
        )
    };
    let out = rotate_point(&car.crash_matrix, [i(0), i(1), i(2)]);
    for (k, e) in out.iter().enumerate() {
        let n = ext_to_int(*e) as u32;
        scene.object_work().object_position[k].set_bits(n);
        slot.place[k].set(n.wrapping_add(car.position[k].get()));
    }
    slot.car_word.set(car.probe_point_face[4].get());
    slot.age.set(car.step_clock.get());
    for k in 0..16 {
        slot.matrix[k].set(car.crash_matrix[k].get());
    }
    let k = work.part_fling.bits();
    slot.motion[0].set_bits(
        int_mul_add(
            scene.object_work().object_position[0].get() as i32,
            k,
            &car.velocity[0],
            None,
        )
        .to_bits(),
    );
    slot.motion[1].set_bits(
        int_mul_add(
            scene.object_work().object_position[1].get() as i32,
            k,
            car.velocity[1].bits(),
            Some(numbers.two.bits()),
        )
        .to_bits(),
    );
    slot.motion[2].set_bits(
        int_mul_add(
            scene.object_work().object_position[2].get() as i32,
            k,
            &car.velocity[2],
            None,
        )
        .to_bits(),
    );
    for k in 0..3 {
        slot.spin[k].set(car.spin[k].get());
    }
    Ok(())
}
