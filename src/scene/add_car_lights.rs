//! `scene::add_car_lights`: which cars' lamps light an object.
//!
//! Calls `scene::add_car_light` for each race car whose lamps light what the drawn view shows,
//! by the road's rule (`race::frame_draw::lamp_cars`, `rc2k_core::race::lights::LampCars`): one
//! car's lamps alone (a network race's own car, a network race's replay followed in one view), or
//! the players' cars but the car being drawn (the drawn-car light cell), every car's with the
//! headlights option "all cars".
//!
//! Refuses no cars or more than the eight blocks.

use crate::errors::{BadInput, Refused};
use crate::race::frame_view::CarView;
use rc2k_core::net::net_race::NetRace;

// ---- game data ----
const BLOCKS: u32 = 8;

// Logic: `rc2k_core::race::lights::LampCars::light`
/// `drawn`: the car whose lights are skipped, or -1 (`ObjectLight::drawn_car_light`); `view`:
/// the drawn view.
pub fn add_car_lights(
    reads: crate::scene::workers::DrawReads<'_>,
    net_race: Option<NetRace>,
    drawn: u32,
    view: u32,
    add: &mut dyn FnMut(&CarView, u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let cars: u32 = reads.view.race_cars();
    if cars == 0 || cars > BLOCKS {
        return Err(BadInput("no cars, or more than their blocks").into());
    }
    let lamps = crate::race::frame_draw::lamp_cars(reads.view, net_race, view)?;
    let all = reads.vars.settings_word(4) == 1;
    for car in 0..cars {
        let reading = crate::scene::reading::Reading::new();
        let lit = lamps.light(
            car,
            all,
            Some(drawn),
            || reads.vars.run_kind(),
            || reading.keep(reads.view.drawn_view_car(view)),
        );
        reading.finish()?;
        if lit {
            // The car as the drawing sees it, and its marks count (the drawing's own, in
            // the block: `race::car_parts`).
            let marks = reads.car_blocks.read(|blocks| {
                let block = crate::race::owner_shared::RaceShared::race_car_or_in(
                    &reads.work.borrow(),
                    blocks,
                    car as usize,
                    "no car",
                )?;
                Ok::<_, BadInput>(blocks.parts_of(reads.car_parts, block).marks_count)
            })?;
            add(
                reads
                    .frame_cars
                    .car(reads.work.block_number(car as usize)?)?,
                marks,
            )?;
        }
    }
    Ok(())
}
