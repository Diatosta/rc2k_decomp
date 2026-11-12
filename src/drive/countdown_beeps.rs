//! `drive::countdown_beeps` (0x440E6C): `drive::countdown_beep` for the two cars of a two-car race
//! (`g_currentCar` 0 and 1, the blocks `g_carBlockStore` + 0 and + 0xFE0, the seconds into
//! `lights` and `unk_6D1492`).
//!
//! Caller: `drive::replay_step`. Refuses what `countdown_beep` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn countdown_beeps(race: &Race, audio: &crate::audio::owner::Audio) -> Result<(), Refused> {
    for (car, to) in [
        (0u32, &race.frame.panel_and_overlay.get().panel.lights[0]),
        (1, &race.frame.panel_and_overlay.get().panel.lights[1]),
    ] {
        race.shared.work.get().current_car.set(car);
        crate::drive::countdown_beep::countdown_beep(
            race,
            &race.car_blocks().cars[car as usize],
            &to.lit,
            audio,
        )?;
    }
    Ok(())
}
