//! `physics::car_collisions` (0x45F615): the work car against the other cars, in race
//! modes 0 and 2 (`dword_62389C`; else nothing). `dword_6D3BB4` = 0; for each car n
//! below `g_raceCars` but the current one (`g_currentCar`), whose block
//! (`g_carBlocks[n]`, into `dword_6D3BB8`, n into `dword_6D3CD4`) has +0x25C set: the
//! places' difference as the place (`scene::set_place`), `g_crashMatrix` x that into
//! `g_turnedMatrix`, that x the block's matrix (+0x9CC) into `flt_6D3C40`
//! (`scene::multiply_matrices`); then for its eight corners (`dword_6D3BE0[k]`,
//! `dword_6D3C00[k]`, `dword_6D3C20[k]` into `dword_6D3BD4` ..) `physics::corner_hits_car`.
//! Every step in `pusha` / `popa`: only ebx, the count, is left.
//!
//! In:  nothing
//! Out: ebx the cars' count (at least 1) in modes 0 and 2; the rest kept; no flags read
//!
//! Callers: `sub_43B635` (every physics step), `sub_43BDD1`. Refuses (before anything) more
//! than eight cars or a null block among them (the original reads past `g_carBlocks` or
//! through null), and what `corner_hits_car` refuses (what was written put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::CAR;
use crate::physics::car_store::Car;
use crate::physics::corner_hits_car::corner_hits_car;
use crate::physics::flow::{a, g, p};
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
use crate::scene::matrix::{multiply_matrices, set_place};

fn rd(x: u32) -> u32 {
    unsafe { (x as *const u32).read_unaligned() }
}

/// The cars looked at: `g_raceCars`, at least one (the loop runs once).
fn cars() -> u32 {
    RACE_VIEW.race_cars.get().max(1)
}

/// The Rust version. The registers it leaves (ebx the cars looked at) are read by none
/// of its callers.
pub unsafe fn car_collisions() -> Result<(), Refused> {
    if !matches!(RACE_VARS.run_kind.get(), 0 | 2) {
        return Ok(());
    }
    let n = cars();
    if n > 8 {
        return Err(BadInput("more than eight cars (g_carBlocks' room)").into());
    }
    if (0..n).any(|k| k != RACE_WORK.current_car.get() && RACE_WORK.car_block(k as usize) == 0) {
        return Err(BadInput("a null car block").into());
    }
    {
        p(0x6D3BB4, 0);
        for car in 0..n {
            if car == RACE_WORK.current_car.get() {
                continue;
            }
            p(0x6D3CD4, car);
            let block = RACE_WORK.car_block(car as usize);
            p(0x6D3BB8, block);
            // SAFETY: a `g_carBlocks` entry of the eight, not null (both checked above).
            if unsafe { Car::at(block) }.driver_kind.bits() == 0 {
                continue;
            }
            unsafe {
                set_place(
                    [0, 4, 8].map(|o| g(0x71A10C + o).wrapping_sub(rd(block + 0x134 + o as u32))),
                );
                multiply_matrices(
                    CAR.crash_matrix[0].addr() as u32 as usize,
                    a(0x5EA04C) as usize,
                    a(0x5EA10C) as usize,
                );
                multiply_matrices(
                    a(0x5EA10C) as usize,
                    (block + 0x9CC) as usize,
                    a(0x6D3C40) as usize,
                );
            }
            for k in 0..8 {
                p(0x6D3BD4, g(0x6D3BE0 + 4 * k));
                p(0x6D3BD8, g(0x6D3C00 + 4 * k));
                p(0x6D3BDC, g(0x6D3C20 + 4 * k));
                unsafe { corner_hits_car(block) }?;
            }
        }
        Ok(())
    }
}
