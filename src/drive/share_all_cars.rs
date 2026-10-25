//! `drive::share_all_cars` (0x456484): `drive::keep_lead_car_values`,
//! `drive::set_listener`, then for each of the eight cars (`g_currentCar` = n)
//! `drive::share_car_state` with its block, its record (`off_71B3D8[n]`) and its sound
//! object (`g_raceSoundObjects[n]`) inside `pusha` / `popa`; then `dword_5C5688` = 1
//! when `dword_70F410` is not 0x21 and the first record's +0x258 is 10 or more (else 0);
//! then `drive::put_back_lead_car_values`.
//!
//! In:  nothing
//! Out: ecx 0, esi the last block, edi the first record, ebp the last sound object, and
//!      eax, ebx, edx as `set_listener` left them (eax the record's +0x258 when
//!      `dword_70F410` is not 0x21); no flags read (its callers go on with `mov`)
//!
//! Callers: race init's loop (`sub_4462B9`, 0x4463F4) and `sub_456336` (0x4563C8). Refuses
//! (before anything) what `set_listener` refuses, a car whose file number is above 0x21,
//! and a null record or block. Its pieces have their own tests on test memory.

use crate::drive::keep_lead_car_values::keep_lead_car_values;
use crate::drive::put_back_lead_car_values::put_back_lead_car_values;
use crate::drive::set_listener::set_listener;
use crate::drive::share_car_state::share_car_state;
use crate::errors::{BadInput, Refused};
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::kind_store::KIND_TABLES;
use crate::race::mud::car_file_number;
use crate::race::work_store::RACE_WORK;
use rc2k_core::drive::sharing;

fn cars() -> [(u32, u32, u32); 8] {
    std::array::from_fn(|n| {
        (
            RACE_WORK.car_block(n),
            RACE_WORK.car_record(n),
            crate::sound::store::RACE_SOUNDS.table[n].get(),
        )
    })
}

/// The refusals, worked out before anything is written.
unsafe fn refusal() -> Option<&'static str> {
    for (block, record, sound) in cars() {
        if block == 0 || record == 0 || sound == 0 {
            return Some("a null car block, record or sound object");
        }
        let n = car_file_number(unsafe { ((block + 0x24) as *const u32).read_unaligned() });
        if let Err(e) = sharing::check_light_record(n) {
            return Some(e.0);
        }
    }
    None
}

/// The Rust version. The registers it leaves are read by no caller (liveness: no read
/// point after `drive::race_init`'s or `race::run_view_marks`' call).
// Converted: `rc2k_core::drive::sharing::{check_light_record, cars_shown}`.
pub unsafe fn share_all_cars() -> Result<(), Refused> {
    unsafe {
        if let Some(why) = refusal() {
            return Err(BadInput(why).into());
        }
        // `set_listener` refuses before writing; nothing is written before it but the
        // lead car's values, which are kept again by the original.
        set_listener()?;
        keep_lead_car_values();
        for (n, (block, record, sound)) in cars().into_iter().enumerate() {
            RACE_WORK.current_car.set(n as u32);
            share_car_state(block, record, sound)?;
        }
        let first_record: u32 = RACE_WORK.car_records[0].bits();
        KIND_TABLES.sound_objects_shown.set_bits(0u32);
        let (shown, _) = sharing::cars_shown(CAR_BLOCKS.cars[0].car_kind.bits(), || {
            ((first_record + 0x258) as *const u32).read_unaligned()
        });
        if shown {
            KIND_TABLES.sound_objects_shown.set_bits(1u32);
        }
        put_back_lead_car_values();
        Ok(())
    }
}
