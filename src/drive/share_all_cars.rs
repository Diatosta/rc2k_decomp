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
use crate::race::marks_settings_store::{MARKS_SETTINGS, MarkCar};
use crate::race::mud::car_file_number;
use crate::race::work_store::RACE_WORK;
use rc2k_core::drive::sharing;

fn cars() -> Result<
    [(
        u32,
        &'static MarkCar,
        &'static crate::sound::store::SoundObject,
    ); 8],
    BadInput,
> {
    let records = RACE_WORK.car_records_resolved()?;
    Ok(std::array::from_fn(|n| {
        (
            RACE_WORK.car_block(n),
            records[n],
            &crate::sound::owner::sound().race.sounds.objects[n],
        )
    }))
}

/// The refusals, worked out before anything is written.
fn refusal() -> Result<Option<&'static str>, BadInput> {
    for (block, _, _) in cars()? {
        if block == 0 {
            return Ok(Some("a null car block or record"));
        }
        let n = car_file_number(crate::physics::car_store::Car::at(block).car_kind.get());
        if let Err(e) = sharing::check_light_record(n) {
            return Ok(Some(e.0));
        }
    }
    Ok(None)
}

/// The Rust version. The registers it leaves are read by no caller (liveness: no read
/// point after `drive::race_init`'s or `race::run_view_marks`' call).
// Converted: `rc2k_core::drive::sharing::{check_light_record, cars_shown}`.
pub fn share_all_cars() -> Result<(), Refused> {
    if let Some(why) = refusal()? {
        return Err(BadInput(why).into());
    }
    // `set_listener` refuses before writing; nothing is written before it but the
    // lead car's values, which are kept again by the original.
    set_listener()?;
    keep_lead_car_values();
    for (n, (block, record, sound)) in cars()?.into_iter().enumerate() {
        RACE_WORK.current_car.set(n as u32);
        share_car_state(crate::physics::car_store::Car::at(block), record, sound)?;
    }
    let first_record = RACE_WORK.car_record(crate::race::blocks_store::own_car_number())?;
    MARKS_SETTINGS.sound_objects_shown.set_bits(0u32);
    let reading = crate::scene::reading::Reading::new();
    let (shown, _) =
        sharing::cars_shown(crate::race::blocks_store::own_car().car_kind.bits(), || {
            reading.keep(first_record.word(0x258))
        });
    reading.finish()?;
    if shown {
        MARKS_SETTINGS.sound_objects_shown.set_bits(1u32);
    }
    put_back_lead_car_values();
    Ok(())
}
