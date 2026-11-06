//! `scene::add_car_lights` (0x44F1FE): which cars' lamps light an object.
//!
//! In:  nothing
//! Out: ecx = the number of cars, esi = the last car's block; everything else preserved
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
use crate::physics::car_store::Car;
use crate::race::owner::race;

// ---- game data ----
const BLOCKS: u32 = 8;

// ---- Rust version ----

/// The game's variables the choice reads.
struct GameCars(crate::scene::reading::Reading);

impl rc2k_core::scene::lamps::CarLights for GameCars {
    fn one_car(&self) -> bool {
        crate::net::owner::net().vars.session.get() != 0
    }
    fn the_car(&self) -> u32 {
        crate::net::owner::net().vars.player.get()
    }
    fn all_cars(&self) -> bool {
        crate::race::owner::race().shared.vars.settings_words[4].bits() == 1
    }
    fn near(&self) -> u32 {
        crate::race::owner::race().shared.view.players.bits()
    }
    fn view(&self) -> u32 {
        crate::race::owner::race().shared.vars.run_kind.bits()
    }
    fn drawn(&self) -> u32 {
        race().physics.ground.drawn_car_light.bits()
    }
    fn viewed(&self) -> u32 {
        // The store holds two views; the original's index is not held to them.
        self.0.keep(crate::session::read_on_in(
            &crate::race::owner::race().shared.view,
            &crate::race::owner::race().shared.view.view_cars,
            crate::race::owner::race().shared.view.drawn_view.bits() as usize,
            "the drawn view's car",
        ))
    }
}

// Converted: `rc2k_core::scene::lamps::car_lights`
pub fn add_car_lights(add: &mut dyn FnMut(&Car) -> Result<(), Refused>) -> Result<(), Refused> {
    {
        let cars: u32 = crate::race::owner::race().shared.view.race_cars.bits();
        if cars == 0 || cars > BLOCKS {
            return Err(BadInput("no cars, or more than their blocks").into());
        }
        let lights = GameCars(crate::scene::reading::Reading::new());
        for car in 0..cars {
            let block = crate::race::owner::race()
                .shared
                .work
                .car_block(car as usize);
            let lit = rc2k_core::scene::lamps::car_lights(&lights, car);
            lights.0.finish()?;
            if lit {
                add(crate::race::blocks_store::block_at(block, "no car")?)?;
            }
        }
        Ok(())
    }
}
