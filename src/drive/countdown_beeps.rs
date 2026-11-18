//! `drive::countdown_beep` for the two cars of a two-car race (and a split screen's third and
//! fourth, whose co-drivers speak). Refuses what `countdown_beep`
//! refuses.

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
    // A split screen's third and fourth players hear the count down in their own voices; their
    // seconds go nowhere (the start lights show the second view's).
    let others = crate::sound::cue_players::count().min(4);
    for car in 2..others {
        race.shared.work.get().current_car.set(car);
        crate::drive::countdown_beep::countdown_beep(
            race,
            &race.car_blocks().cars[car as usize],
            &crate::memory::session::PlainDword::new(0),
            audio,
        )?;
    }
    if others > 2 {
        race.shared.work.get().current_car.set(1);
    }
    Ok(())
}
