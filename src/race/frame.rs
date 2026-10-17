//! The race's frame outside the physics (`sub_444D46`'s callees, IDA 2026-10-17): the
//! players' states, the cars' counters, the view cars loaded into the work copy. Here
//! its leaves.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::race::objects::CAR_BLOCKS_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at};

const ONE_CHOSEN_RVA: usize = rva(0x52A910); // dword_52A910
const CHOSEN_CAR_RVA: usize = rva(0x52A920); // dword_52A920
pub const PLAYERS_RVA: usize = rva(0x63DB94); // dword_63DB94

fn block(n: u32) -> u32 {
    unsafe { game_read(CAR_BLOCKS_RVA + 4 * n as usize) }
}

/// `race::players_all_out` (0x445506): carry when every player is out (+0xB0 not 0): the
/// chosen car's with `dword_52A910` 1, else each of the `dword_63DB94` players' (none: no
/// carry). Every register kept. Refuses, before anything, a chosen car past 7 or more
/// than 8 players (the original reads past the table).
pub unsafe fn players_all_out() -> Result<bool, Refused> {
    unsafe {
        let out = |n: u32| ((block(n) + 0xB0) as *const u32).read_unaligned() != 0;
        if game_read::<u32>(ONE_CHOSEN_RVA) == 1 {
            let n: u32 = game_read(CHOSEN_CAR_RVA);
            if n > 7 {
                return Err(BadInput("a chosen car past the eight").into());
            }
            return Ok(out(n));
        }
        let players: u32 = game_read(PLAYERS_RVA);
        if players > 8 {
            return Err(BadInput("more than eight players").into());
        }
        if players == 0 {
            return Ok(false);
        }
        Ok((0..players).all(out))
    }
}

/// `race::set_car_counter` (0x43A96C): block esi's +0x71C eax, +0x730 0, +0x72C 0x12C.
pub unsafe fn set_car_counter(at: u32, value: u32) {
    unsafe {
        ((at + 0x71C) as *mut u32).write_unaligned(value);
        ((at + 0x730) as *mut u32).write_unaligned(0);
        ((at + 0x72C) as *mut u32).write_unaligned(0x12C);
    }
}

/// `race::set_both_counters` (0x43A987): `set_car_counter` of the first two blocks (by
/// their fixed addresses), each with eax.
pub unsafe fn set_both_counters(value: u32) {
    for b in [0x70F3EC, 0x7103CC] {
        unsafe { set_car_counter(game_addr(rva(b)) as u32, value) };
    }
}

pub fn counter_regions(b: u32) -> Vec<Region> {
    vec![(b + 0x71C, 0x18)]
}

pub fn both_counter_regions() -> Vec<Region> {
    [0x70F3EC, 0x7103CC]
        .iter()
        .flat_map(|&b| counter_regions(game_addr(rva(b)) as u32))
        .collect()
}

/// The view cars' numbers: `dword_63DBA8`, `dword_63DBAC`, `dword_63DBA0`.
pub const VIEW_CARS: [usize; 3] = [rva(0x63DBA8), rva(0x63DBAC), rva(0x63DBA0)];

/// `race::load_car_a` / `_b` / `_c` (0x45422F, 0x454241, 0x454277): `g_currentCar` the
/// view car's number, `drive::load_car`. Leaves the registers as it does.
pub unsafe fn load_view_car(n: usize) -> Result<Left, Refused> {
    unsafe {
        game_write(
            crate::drive::car::CURRENT_CAR_RVA,
            game_read::<u32>(VIEW_CARS[n]),
        );
        crate::drive::load_car::load_car()
    }
}

/// What a view car's load writes: the work copy and `g_currentCar`.
pub fn load_regions() -> Vec<Region> {
    vec![
        (
            game_addr(crate::drive::car::CAR_RVA) as u32,
            crate::drive::car::CAR_LEN,
        ),
        at(crate::drive::car::CURRENT_CAR_RVA),
    ]
}
