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
use crate::framework::{game_read, rva};

// ---- game data ----

const CARS_RVA: usize = rva(0x63DB90); // dword_63DB90
const BLOCKS_RVA: usize = rva(0x71B3B8); // g_carBlocks
const BLOCKS: u32 = 8;
const ALL_CARS_RVA: usize = rva(0x6238B8); // dword_6238B8
const NEAR_CARS_RVA: usize = rva(0x63DB94); // dword_63DB94
pub const DRAWN_CAR_RVA: usize = rva(0x639968); // dword_639968
const VIEW_RVA: usize = rva(0x62389C); // dword_62389C
const VIEW_CAR_RVA: usize = rva(0x63DB9C); // dword_63DB9C
const VIEW_CARS_RVA: usize = rva(0x63DBA8); // dword_63DBA8

// ---- Rust version ----

/// The game's variables the choice reads.
struct GameCars;

impl rc2k_core::scene::lamps::CarLights for GameCars {
    fn one_car(&self) -> bool {
        crate::net::store::NET.session.get() != 0
    }
    fn the_car(&self) -> u32 {
        crate::net::store::NET.player.get()
    }
    fn all_cars(&self) -> bool {
        unsafe { game_read::<u32>(ALL_CARS_RVA) == 1 }
    }
    fn near(&self) -> u32 {
        unsafe { game_read(NEAR_CARS_RVA) }
    }
    fn view(&self) -> u32 {
        unsafe { game_read(VIEW_RVA) }
    }
    fn drawn(&self) -> u32 {
        unsafe { game_read(DRAWN_CAR_RVA) }
    }
    fn viewed(&self) -> u32 {
        unsafe {
            game_read(VIEW_CARS_RVA.wrapping_add((game_read::<u32>(VIEW_CAR_RVA) as usize) << 2))
        }
    }
}

// Converted: `rc2k_core::scene::lamps::car_lights`
pub unsafe fn add_car_lights(
    add: &mut dyn FnMut(usize) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let cars: u32 = game_read(CARS_RVA);
        if cars == 0 || cars > BLOCKS {
            return Err(BadInput("no cars, or more than their blocks").into());
        }
        for car in 0..cars {
            let block = game_read::<u32>(BLOCKS_RVA + car as usize * 4);
            if rc2k_core::scene::lamps::car_lights(&GameCars, car) {
                add(block as usize)?;
            }
        }
        Ok(())
    }
}
