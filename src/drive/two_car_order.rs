//! The two cars' places in a two-car race (the car blocks' `two_car_place`): -1 for both outside mode 0
//! (`run_kind`) or with one car; left as they are while either has `finish_flag` set; else 1 for the
//! one ahead (`reset_checkpoint`, then `checkpoint_offset[1]`), 2 for the other. Gives the last figure compared.

use crate::race::owner::Race;
use rc2k_core::drive::places;

pub fn two_car_order(race: &Race) -> Result<(), crate::errors::BadInput> {
    // The figures of car block `car`.
    let figure = |car: usize, o: u32| {
        let block = &race.car_blocks().cars[car];
        match o {
            places::FINISHED => block.finish_flag.get(),
            places::CHECKPOINT => block.reset_checkpoint.get(),
            _ => block.checkpoint_offset[1].get(), // places::TIE_BREAK
        }
    };
    let players = race.shared.view.get().players.get();
    if race.shared.vars.get().run_kind.get() == 0 && players > 2 {
        // A split screen of three or four: every player's place.
        if let Some(order) = places::many_car_order(players as usize, figure) {
            for (car, place) in order.into_iter().enumerate() {
                race.car_blocks().cars[car].two_car_place.set(place);
            }
        }
        return Ok(());
    }
    let order = places::two_car_order(
        race.shared.vars.get().run_kind.get(),
        race.shared.view.get().players.get(),
        figure,
        0,
    );
    if let Some([x, y]) = order.places {
        race.car_blocks().cars[0].two_car_place.set(x);
        race.car_blocks().cars[1].two_car_place.set(y);
    }
    Ok(())
}
