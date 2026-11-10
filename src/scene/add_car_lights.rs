//! `scene::add_car_lights` (0x44F1FE): which cars' lamps light an object.
//!
//! Returns: the number of cars, the last car's block
//!
//! Goes through the `dword_63DB90` blocks of `g_carBlocks` and calls
//! `scene::add_car_light` for:
//! - with `dword_52A910` set: the car `dword_52A920`, or every car while
//!   `dword_6238B8` is 1;
//! - else: the cars under `dword_63DB94` (every car while `dword_6238B8` is 1), but
//!   not the car `dword_639968` (the one being drawn, by its use), and of those all
//!   of them if `dword_63DB94` is 1 or `dword_62389C` is 0 or 2, else only the car
//!   `dword_63DBA8[dword_63DB9C]`.
//!
//! One caller, `scene::gather_lights`. Refuses no cars (the original goes through 2^32) or more
//! than the eight blocks.

use crate::errors::{BadInput, Refused};
use crate::net::owner::net;
use crate::race::frame_view::CarView;
use crate::race::owner::Race;

// ---- game data ----
const BLOCKS: u32 = 8;

// ---- Rust version ----

/// The game's variables the choice reads.
struct GameCars<'a>(&'a Race, crate::scene::reading::Reading, u32, u32);

impl rc2k_core::scene::lamps::CarLights for GameCars<'_> {
    fn one_car(&self) -> bool {
        net().vars.session.get() != 0
    }
    fn the_car(&self) -> u32 {
        net().vars.player.get()
    }
    fn all_cars(&self) -> bool {
        self.0.shared.vars.settings_words[4].bits() == 1
    }
    fn near(&self) -> u32 {
        self.0.shared.view.players.bits()
    }
    fn view(&self) -> u32 {
        self.0.shared.vars.run_kind.bits()
    }
    fn drawn(&self) -> u32 {
        self.2
    }
    fn viewed(&self) -> u32 {
        let shared = &self.0.shared;
        // The store holds two views; the original's index is not held to them.
        self.1.keep(crate::session::read_on_in(
            &shared.view,
            &shared.view.view_cars,
            self.3 as usize,
            "the drawn view's car",
        ))
    }
}

// Converted: `rc2k_core::scene::lamps::car_lights`
/// `drawn`: the car whose lights are skipped, or -1 (`ObjectLight::drawn_car_light`).
pub fn add_car_lights(
    race: &Race,
    drawn: u32,
    view: u32,
    add: &mut dyn FnMut(&CarView, u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let shared = &race.shared;
    {
        let cars: u32 = shared.view.race_cars.bits();
        if cars == 0 || cars > BLOCKS {
            return Err(BadInput("no cars, or more than their blocks").into());
        }
        let lights = GameCars(race, crate::scene::reading::Reading::new(), drawn, view);
        for car in 0..cars {
            let block = shared.work.car_block(car as usize);
            let lit = rc2k_core::scene::lamps::car_lights(&lights, car);
            lights.1.finish()?;
            if lit {
                // The car as the drawing sees it, and its marks count (the drawing's own, in
                // the block: `race::car_parts`).
                let marks = crate::race::blocks_store::block_at(block, "no car")?
                    .marks_count
                    .bits();
                add(crate::race::frame_view::car_at(block)?, marks)?;
            }
        }
        Ok(())
    }
}
