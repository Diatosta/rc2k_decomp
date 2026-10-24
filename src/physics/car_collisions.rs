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
use crate::physics::corner_hits_car::corner_hits_car;
use crate::physics::flow::{a, g, p};
use crate::scene::matrix::{multiply_matrices, set_place};

const BLOCKS: usize = 0x71B3B8; // g_carBlocks (eight)
const CARS: usize = 0x63DB90; // g_raceCars
const CURRENT: usize = 0x71B3F8; // g_currentCar

fn rd(x: u32) -> u32 {
    unsafe { (x as *const u32).read_unaligned() }
}

/// The cars looked at: `g_raceCars`, at least one (the loop runs once).
fn cars() -> u32 {
    g(CARS).max(1)
}

/// The Rust version. The registers it leaves (ebx the cars looked at) are read by none
/// of its callers.
pub unsafe fn car_collisions() -> Result<(), Refused> {
    if !matches!(g(0x62389C), 0 | 2) {
        return Ok(());
    }
    let n = cars();
    if n > 8 {
        return Err(BadInput("more than eight cars (g_carBlocks' room)").into());
    }
    if (0..n).any(|k| k != g(CURRENT) && g(BLOCKS + 4 * k as usize) == 0) {
        return Err(BadInput("a null car block").into());
    }
    {
        p(0x6D3BB4, 0);
        for car in 0..n {
            if car == g(CURRENT) {
                continue;
            }
            p(0x6D3CD4, car);
            let block = g(BLOCKS + 4 * car as usize);
            p(0x6D3BB8, block);
            if rd(block + 0x25C) == 0 {
                continue;
            }
            unsafe {
                set_place(
                    [0, 4, 8].map(|o| g(0x71A10C + o).wrapping_sub(rd(block + 0x134 + o as u32))),
                );
                multiply_matrices(
                    a(0x71A08C) as usize,
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
