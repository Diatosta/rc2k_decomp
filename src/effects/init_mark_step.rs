//! `effects::init_mark_step` (0x456EA7): the stage loading's "init mark" step: the race's cars
//! go into the effects settings' car list, the own car's object (0x50 but in a network race),
//! a switch 1, and the work car's file number (`race::car_file_number` of the work car's kind);
//! for each of the player-count cars (in a network race every car) its record's places +0x23C
//! (0x50 + 0x14 a car) and +0x240 (0xA more), seven values from its kind's row of the light
//! records (the kind its block's +0x24 held to 0 .. 0x16, rows of 0x134) and the seventeen
//! fixed numbers +0x36C .. +0x3AC; then `effects::init_marks` with the settings.
//!
//! Caller: `race::loadlevel_init` (with the log off round it). The file number is
//! worked out by `race::car_file_number`, also `init_marks`' seed; then `init_marks`. Refuses a car count of 0
//! or past 8 (the original's loop would run on) and a missing block or record, before
//! anything.

use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::race::owner::Race;
// Converted: `rc2k_core::effects::race::{check_cars, kind_row, car_fields}`
pub fn init_mark_step(
    race: &Race,
    effects: &Effects,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
    net: &crate::net::owner::Net,
) -> Result<(), Refused> {
    use rc2k_core::effects::race;
    {
        // In a network race `g_players` is 1 but this machine's car may be any: every car's
        // record is needed, or a joined machine's own has no marks (the original's slip).
        let count: u32 = if net.vars.session.get() == 1 {
            race.shared.view.race_cars.bits()
        } else {
            race.shared.view.players.bits()
        };
        let records = race.shared.work.car_records_resolved()?;
        // `check_cars` holds the count to 8 before `record` is asked.
        let record = |n: u32| records[n as usize];
        race::check_cars(count, |n| race.shared.work.has_car_block(n as usize))?;
        race.effects_settings()
            .cars_copy
            .set(race.shared.view.race_cars.get());
        // The one object the effects' shapes are made from: the player's car (the original's
        // fixed 0x50 is the first car, not a joined machine's).
        let own = crate::race::blocks_store::own_car_number() as u32;
        race.effects_settings()
            .first_object
            .set_bits(0x50 + 0x14 * own);
        race.effects_settings().object_count.set_bits(1u32);
        let file = crate::race::mud::car_file_number(
            race.kind_tables(),
            crate::race::blocks_store::own_car().car_kind.bits(),
        );
        race.effects_settings().mark_car_file_number.set_bits(file);
        for n in 0..count {
            let kind = race.shared.work.car_or_end(n as usize).car_kind.get() as i32;
            // Rows of 0x134 bytes, from the sixth group of the kind's light record on, run on
            // past it as the original does.
            let row = (0x6C + race::kind_row(kind)) as u32 as usize;
            let bad = crate::effects::Sticky::default();
            // Each field is a dword at a byte offset of the row, a run-on as the original.
            let fields = race::car_fields(n, |offset| {
                bad.take(crate::pointers::load_value::<u32>(
                    &race.light_records().records,
                    row + offset,
                ))
            });
            bad.done()?;
            let car = record(n);
            car.object.set(fields.object);
            car.second_object.set(fields.second_object);
            for (cell, v) in car.slot_kinds.iter().zip(fields.slot_kinds) {
                cell.set(v);
            }
            for (cell, v) in car.kinds.iter().zip(fields.kinds) {
                cell.set(v);
            }
        }
        // The file number is also `init_marks`' seed.
        crate::effects::init_marks::init_marks(race, effects, file, scene, gfx, system)?;
    }
    Ok(())
}
