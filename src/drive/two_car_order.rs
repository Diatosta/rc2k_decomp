//! `drive::two_car_order` (0x460CDA): the two cars' places in a two-car race (the car blocks'
//! +0xFA4, `g_carBlockStore`): -1 for both outside mode 0 (`run_kind`) or with one car
//! (`players` 1); left as they are while either has +0xA8 set; else 1 for the one ahead
//! (checkpoint +0x94, then +0x624, each by the sign of the `cmp`), 2 for the other.
//!
//! Returns: the last figure compared
//!
//! Caller: `drive::replay_step`.

use crate::race::owner::Race;
use rc2k_core::drive::places;

// Converted: `rc2k_core::drive::places::two_car_order`.
pub fn two_car_order(race: &Race) -> Result<(), crate::errors::BadInput> {
    // g_carBlockStore: the figures of car block `car`.
    let figure = |car: usize, o: u32| {
        let block = &race.car_blocks().cars[car];
        match o {
            places::FINISHED => block.finish_flag.get(),
            places::CHECKPOINT => block.reset_checkpoint.get(),
            _ => block.checkpoint_offset[1].get(), // places::TIE_BREAK
        }
    };
    let order = places::two_car_order(
        race.shared.vars.run_kind.get(),
        race.shared.view.players.get(),
        figure,
        0,
    );
    if let Some([x, y]) = order.places {
        race.car_blocks().cars[0].two_car_place.set(x);
        race.car_blocks().cars[1].two_car_place.set(y);
    }
    Ok(())
}
