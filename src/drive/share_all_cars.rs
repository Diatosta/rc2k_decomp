//! `drive::share_all_cars` (0x456484): `drive::keep_lead_car_values`, `drive::set_listener`, then
//! for each of the eight cars (`g_currentCar` = n) `drive::share_car_state` with its block, its
//! record (`car_records`[n]`) and its sound object (`g_raceSoundObjects[n]`); then `sound_objects_shown` = 1
//! when `CarBlocks.cars[0].car_kind` is not 0x21 and the first record's +0x258 is 10 or more (else 0); then
//! `drive::put_back_lead_car_values`.
//!
//! Callers: race init's loop (`drive::race_init`, 0x4463F4) and `race::run_view_marks`. Refuses
//! (before anything) what `set_listener` refuses, a car whose file number is above 0x21, and a null
//! record or block. Its pieces have their own tests on test memory.

use crate::drive::keep_lead_car_values::keep_lead_car_values;
use crate::drive::put_back_lead_car_values::put_back_lead_car_values;
use crate::drive::set_listener::set_listener;
use crate::drive::share_car_state::share_car_state;
use crate::errors::{BadInput, Refused};
use crate::race::effects_settings_store::CarEffects;
use crate::race::mud::car_file_number;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::sound::owner::Sound;
use rc2k_core::drive::sharing;

fn cars(
    shared: &RaceShared,
    sound: &'static Sound,
) -> Result<
    [(
        usize,
        &'static CarEffects,
        &'static crate::sound::store::SoundObject,
    ); 8],
    BadInput,
> {
    let records = shared.car_records()?;
    Ok(std::array::from_fn(|n| {
        (n, records[n], &sound.race.sounds.objects[n])
    }))
}

/// The refusals, worked out before anything is written.
fn refusal(race: &Race, sound: &'static Sound) -> Result<Option<&'static str>, BadInput> {
    let shared = &race.shared;
    for (block, _, _) in cars(shared, sound)? {
        if !shared.work.get().has_car_block(block) {
            return Ok(Some("a null car block or record"));
        }
        let n = car_file_number(
            race.kind_tables(),
            shared.race_car_or_end(block).car_kind.get(),
        );
        if let Err(e) = sharing::check_light_record(n) {
            return Ok(Some(e.0));
        }
    }
    Ok(None)
}

// Converted: `rc2k_core::drive::sharing::{check_light_record, cars_shown}`.
pub fn share_all_cars(
    race: &Race,
    sound: &'static Sound,
    scene: &crate::scene::owner::Scene,
    net: &crate::net::owner::Net,
    view: crate::drive::set_listener::ListenerView<'_>,
) -> Result<(), Refused> {
    if let Some(why) = refusal(race, sound)? {
        return Err(BadInput(why).into());
    }
    // `set_listener` refuses before writing; nothing is written before it but the
    // lead car's values, which are kept again by the original.
    set_listener(race, scene, net, view)?;
    keep_lead_car_values(race.shared.own_car(net.vars.session.get()), &race.frame);
    for (n, (block, record, sound)) in cars(&race.shared, sound)?.into_iter().enumerate() {
        race.shared.work.get().current_car.set(n as u32);
        share_car_state(race, race.shared.race_car_or_end(block), record, sound)?;
    }
    let first_record = race
        .shared
        .car_record(race.shared.own_car_number(net.vars.session.get()))?;
    race.effects_settings().sound_objects_shown.set(0u32);
    let reading = crate::scene::reading::Reading::new();
    let (shown, _) = sharing::cars_shown(
        race.shared.own_car(net.vars.session.get()).car_kind.get(),
        || reading.keep(Ok(first_record.face_sums[0].get())),
    );
    reading.finish()?;
    if shown {
        race.effects_settings().sound_objects_shown.set(1u32);
    }
    put_back_lead_car_values(race.shared.own_car(net.vars.session.get()), &race.frame);
    Ok(())
}
