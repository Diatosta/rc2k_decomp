//! `scene::add_car_lights`: which cars' lamps light an object.
//!
//! Returns the number of cars and the last car's block. Calls `scene::add_car_light` for:
//! - in a network session: the network player's car, or every car while the split-screen
//!   setting is 1;
//! - else: the cars under the player count (every car while the split-screen setting is 1),
//!   but not the car being drawn (the drawn-car light cell), and of those all of them if
//!   the player count is 1 or the run kind is 0 or 2, else only the drawn view's car.
//!
//! Refuses no cars or more than the eight blocks.

use crate::errors::{BadInput, Refused};
use crate::net::owner::Net;
use crate::race::frame_view::CarView;

// ---- game data ----
const BLOCKS: u32 = 8;

/// The game's variables the choice reads.
struct GameCars<'a>(
    crate::scene::reading::Reading,
    u32,
    u32,
    &'a Net,
    crate::workers::DrawReads<'a>,
);

impl rc2k_core::scene::lamps::CarLights for GameCars<'_> {
    fn one_car(&self) -> bool {
        self.3.vars.session.get() != 0
    }
    fn the_car(&self) -> u32 {
        self.3.vars.player.get()
    }
    fn all_cars(&self) -> bool {
        self.4.vars.settings_words[4].get() == 1
    }
    fn near(&self) -> u32 {
        self.4.view.players.get()
    }
    fn view(&self) -> u32 {
        self.4.vars.run_kind.get()
    }
    fn drawn(&self) -> u32 {
        self.1
    }
    fn viewed(&self) -> u32 {
        // The drawn view is 0 or 1: a part's own view number modulo 2, or a players'
        // scene's, which only `race::draw_race_scene` writes (0, or the player 0 .. 1).
        self.0.keep(self.4.view.drawn_view_car(self.2))
    }
}

// Logic: `rc2k_core::scene::lamps::car_lights`
/// `drawn`: the car whose lights are skipped, or -1 (`ObjectLight::drawn_car_light`).
pub fn add_car_lights(
    reads: crate::workers::DrawReads<'_>,
    net: &Net,
    drawn: u32,
    view: u32,
    add: &mut dyn FnMut(&CarView, u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    {
        let cars: u32 = reads.view.race_cars.get();
        if cars == 0 || cars > BLOCKS {
            return Err(BadInput("no cars, or more than their blocks").into());
        }
        let lights = GameCars(
            crate::scene::reading::Reading::new(),
            drawn,
            view,
            net,
            reads,
        );
        for car in 0..cars {
            let lit = rc2k_core::scene::lamps::car_lights(&lights, car);
            lights.0.finish()?;
            if lit {
                // The car as the drawing sees it, and its marks count (the drawing's own, in
                // the block: `race::car_parts`).
                let marks = crate::race::owner_shared::RaceShared::race_car_or_in(
                    reads.work,
                    reads.car_blocks,
                    car as usize,
                    "no car",
                )?
                .marks_count
                .get();
                add(
                    crate::race::frame_view::car(reads.work.block_number(car as usize)?)?,
                    marks,
                )?;
            }
        }
        Ok(())
    }
}
