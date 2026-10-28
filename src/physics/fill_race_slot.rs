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
use crate::race::numbers_store::NUMBERS;
use crate::race::work_store::RACE_WORK;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::rotate_point::rotate_point;
use rc2k_core::physics::parts::{LAST_PART, choose_slot};

fn at(a: u32) -> u32 {
    crate::pointers::read_or_end::<u32>(a, "physics::fill_race_slot's operand")
}

fn put(a: u32, v: u32) {
    crate::pointers::write_or_end::<u32>(a, v, "physics::fill_race_slot's operand")
}

/// The Rust version: part `part` (eax) into a race slot. The registers the original leaves
/// are read by none of its callers.
// Converted: `rc2k_core::physics::parts::choose_slot` (the slot; the floats stay x87).
pub fn fill_race_slot(part: u32) -> Result<(), Refused> {
    if part > LAST_PART {
        return Err(BadInput("a part past 0xFF").into());
    }
    let first = RACE_WORK.race_slots[0].part.addr();
    let slot_at = |c: usize| first + 0x78 * c as u32;
    let Some(choice) = choose_slot(|c| at(slot_at(c)), |c| at(slot_at(c) + 0x64)) else {
        return Err(BadInput("no race slot to take (every +0x64 -1)").into());
    };
    let s = slot_at(choice.slot);
    put(s, part);
    let i = |t: usize| {
        int_80(
            crate::scene::object_slots_store::OBJECT_SLOTS.part_extents[t][part as usize].get()
                as i32,
            false,
        )
    };
    let out = rotate_point(&CAR.crash_matrix, [i(0), i(1), i(2)]);
    for (k, e) in out.iter().enumerate() {
        let n = ext_to_int(*e) as u32;
        OBJECT_WORK.object_position[k].set_bits(n);
        put(s + 4 + 4 * k as u32, n.wrapping_add(CAR.position[k].get()));
    }
    put(s + 0x60, CAR.probe_point_face[4].get());
    put(s + 0x64, CAR.step_clock.get());
    for k in 0..16u32 {
        put(s + 0x10 + 4 * k, CAR.crash_matrix[k as usize].bits());
    }
    let k = RACE_WORK.part_fling.bits();
    put(
        s + 0x50,
        int_mul_add(
            OBJECT_WORK.object_position[0].get() as i32,
            k,
            CAR.velocity[0].bits(),
            None,
        ),
    );
    put(
        s + 0x54,
        int_mul_add(
            OBJECT_WORK.object_position[1].get() as i32,
            k,
            CAR.velocity[1].bits(),
            Some(NUMBERS.two.bits()),
        ),
    );
    put(
        s + 0x58,
        int_mul_add(
            OBJECT_WORK.object_position[2].get() as i32,
            k,
            CAR.velocity[2].bits(),
            None,
        ),
    );
    for k in 0..3u32 {
        put(s + 0x68 + 4 * k, CAR.spin[k as usize].bits());
    }
    Ok(())
}
