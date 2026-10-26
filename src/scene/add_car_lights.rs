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
use crate::physics::ground_store::GROUND_WORK;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;

// ---- game data ----
const BLOCKS: u32 = 8;

// ---- Rust version ----

/// The game's variables the choice reads.
struct GameCars(crate::scene::reading::Reading);

impl rc2k_core::scene::lamps::CarLights for GameCars {
    fn one_car(&self) -> bool {
        crate::net::store::NET.session.get() != 0
    }
    fn the_car(&self) -> u32 {
        crate::net::store::NET.player.get()
    }
    fn all_cars(&self) -> bool {
        RACE_VARS.settings_words[4].bits() == 1
    }
    fn near(&self) -> u32 {
        RACE_VIEW.players.bits()
    }
    fn view(&self) -> u32 {
        RACE_VARS.run_kind.bits()
    }
    fn drawn(&self) -> u32 {
        GROUND_WORK.drawn_car_light.bits()
    }
    fn viewed(&self) -> u32 {
        // The store holds two views; the original's index is not held to them.
        self.0.read(
            (RACE_VIEW.view_cars[0].addr()).wrapping_add(RACE_VIEW.drawn_view.bits() << 2),
            "the drawn view's car",
        )
    }
}

// Converted: `rc2k_core::scene::lamps::car_lights`
pub fn add_car_lights(add: &mut dyn FnMut(usize) -> Result<(), Refused>) -> Result<(), Refused> {
    {
        let cars: u32 = RACE_VIEW.race_cars.bits();
        if cars == 0 || cars > BLOCKS {
            return Err(BadInput("no cars, or more than their blocks").into());
        }
        let lights = GameCars(crate::scene::reading::Reading::new());
        for car in 0..cars {
            let block = RACE_WORK.car_block(car as usize);
            let lit = rc2k_core::scene::lamps::car_lights(&lights, car);
            lights.0.finish()?;
            if lit {
                add(block as usize)?;
            }
        }
        Ok(())
    }
}
