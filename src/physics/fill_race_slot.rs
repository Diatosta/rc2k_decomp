//! `physics::fill_race_slot` (0x462263): a loose part (eax: its number) put into one of
//! the eight `g_raceSlots` (0x78 bytes each): the first empty one (+0 0), else the one
//! with the lowest +0x64 (unsigned, the first of equals; with every +0x64 -1 the original
//! takes ebp as it came: refused). There: +0 the part; its middle (`g_partMiddleX` /
//! `Y` / `Z`[part], whole numbers) turned by `g_crashMatrix` (`scene::rotate_point`),
//! `fistp` into `dword_635004` .. and plus the car's place into +4 .. +0xC; +0x60
//! `g_car` +0xE00, +0x64 `g_car` +8; the car's matrix (16 dwords) at +0x10; +0x50 ..
//! +0x58 the turned middle x `flt_71BE8C` plus the car's motion (+2.0 for y); +0x68 ..
//! +0x70 the car's spin. A reading: a part breaking off, flying with the car's motion.
//!
//! In:  eax the part
//! Out: eax `g_car` +8, ebx the part, ecx the slot's number (8 when none was empty),
//!      edx -1 or the lowest +0x64, esi the slot, ebp as the search leaves it; no flags read
//!
//! Caller: `physics::fill_slot_on_bit`. Refuses (before anything) a part past 0xFF, or no
//! slot to take.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::x87_run::{ext_to_int, int_80, int_mul_add};
use crate::physics::car_store::CAR;
use crate::race::owner_course::Course;
use crate::race::work_store::RACE_WORK;
use crate::scene::rotate_point::rotate_point;
use rc2k_core::physics::parts::{LAST_PART, choose_slot};

/// The Rust version: part `part` (eax) into a race slot. The registers the original leaves
/// are read by none of its callers.
// Converted: `rc2k_core::physics::parts::choose_slot` (the slot; the floats stay x87).
pub fn fill_race_slot(course: &Course, part: u32) -> Result<(), Refused> {
    if part > LAST_PART {
        return Err(BadInput("a part past 0xFF").into());
    }
    let Some(choice) = choose_slot(
        |c| RACE_WORK.race_slots[c].part.get(),
        |c| RACE_WORK.race_slots[c].age.get(),
    ) else {
        return Err(BadInput("no race slot to take (every +0x64 -1)").into());
    };
    let slot = &RACE_WORK.race_slots[choice.slot];
    slot.part.set(part);
    let i = |t: usize| {
        int_80(
            crate::scene::owner::scene().objects.slots.part_extents[t][part as usize].get() as i32,
            false,
        )
    };
    let out = rotate_point(&CAR.crash_matrix, [i(0), i(1), i(2)]);
    for (k, e) in out.iter().enumerate() {
        let n = ext_to_int(*e) as u32;
        crate::scene::owner::scene()
            .drawing
            .object_work
            .object_position[k]
            .set_bits(n);
        slot.place[k].set(n.wrapping_add(CAR.position[k].get()));
    }
    slot.car_word.set(CAR.probe_point_face[4].get());
    slot.age.set(CAR.step_clock.get());
    for k in 0..16 {
        slot.matrix[k].set_bits(CAR.crash_matrix[k].bits());
    }
    let k = RACE_WORK.part_fling.bits();
    slot.motion[0].set_bits(int_mul_add(
        crate::scene::owner::scene()
            .drawing
            .object_work
            .object_position[0]
            .get() as i32,
        k,
        CAR.velocity[0].bits(),
        None,
    ));
    slot.motion[1].set_bits(int_mul_add(
        crate::scene::owner::scene()
            .drawing
            .object_work
            .object_position[1]
            .get() as i32,
        k,
        CAR.velocity[1].bits(),
        Some(course.numbers.two.bits()),
    ));
    slot.motion[2].set_bits(int_mul_add(
        crate::scene::owner::scene()
            .drawing
            .object_work
            .object_position[2]
            .get() as i32,
        k,
        CAR.velocity[2].bits(),
        None,
    ));
    for k in 0..3 {
        slot.spin[k].set_bits(CAR.spin[k].bits());
    }
    Ok(())
}
