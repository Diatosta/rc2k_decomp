//! `scene::add_car_lights` (0x44F1FE): which cars' lamps light an object.
//!
//! Returns: the number of cars, the last car's block
//!
//! Goes through the race's cars (`race_cars`) and calls `scene::add_car_light` for:
//! - in a network session: the network player's car, or every car while the split-screen
//!   setting is 1;
//! - else: the cars under the player count (every car while the split-screen setting is 1),
//!   but not the car being drawn (the drawn-car light cell), and of those all of them if
//!   the player count is 1 or the run kind is 0 or 2, else only the drawn view's car.
//!
//! One caller, `scene::gather_lights`. Refuses no cars (the original goes through 2^32) or more
//! than the eight blocks.

use crate::errors::{BadInput, Refused};
use crate::net::owner::Net;
use crate::race::frame_view::CarView;
use crate::race::owner::Race;

// ---- game data ----
const BLOCKS: u32 = 8;

// ---- Rust version ----

/// The game's variables the choice reads.
struct GameCars<'a>(&'a Race, crate::scene::reading::Reading, u32, u32, &'a Net);

impl rc2k_core::scene::lamps::CarLights for GameCars<'_> {
    fn one_car(&self) -> bool {
        self.4.vars.session.get() != 0
    }
    fn the_car(&self) -> u32 {
        self.4.vars.player.get()
    }
    fn all_cars(&self) -> bool {
        self.0.shared.vars.settings_words[4].get() == 1
    }
    fn near(&self) -> u32 {
        self.0.shared.view.players.get()
    }
    fn view(&self) -> u32 {
        self.0.shared.vars.run_kind.get()
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
    net: &Net,
    drawn: u32,
    view: u32,
    add: &mut dyn FnMut(&CarView, u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let shared = &race.shared;
    {
        let cars: u32 = shared.view.race_cars.get();
        if cars == 0 || cars > BLOCKS {
            return Err(BadInput("no cars, or more than their blocks").into());
        }
        let lights = GameCars(
            race,
            crate::scene::reading::Reading::new(),
            drawn,
            view,
            net,
        );
        for car in 0..cars {
            let lit = rc2k_core::scene::lamps::car_lights(&lights, car);
            lights.1.finish()?;
            if lit {
                // The car as the drawing sees it, and its marks count (the drawing's own, in
                // the block: `race::car_parts`).
                let marks = shared
                    .work
                    .block_car(car as usize, "no car")?
                    .marks_count
                    .get();
                add(
                    crate::race::frame_view::car(shared.work.block_number(car as usize)?)?,
                    marks,
                )?;
            }
        }
        Ok(())
    }
}
