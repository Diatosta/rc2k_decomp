//! `scene::set_up_engine` (0x442AF6): the 3D code's tables, and the cars' figures.
//!
//! In:  nothing
//! Out: as `sub_4217E4` leaves them
//!
//! `scene::fill_engine_tables`, then `sub_4217E4` (the cars' physics run for the
//! figures of the car specification screen, by the look of it; not ported,
//! `call_game`). 22 call sites, among them `menu::main_menu_loop` (twice) and
//! `menu::play_films`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, rva};
use crate::scene::fill_engine_tables::fill_engine_tables;

// scene::fill_engine_tables
const CARS_RVA: usize = rva(0x4217E4); // sub_4217E4
// ---- Rust version ----

/// `step` stands for the two calls: 0 the tables, 1 the cars.
pub fn set_up_engine(r: &mut Regs, step: &mut dyn FnMut(u32, &mut Regs)) {
    step(0, r);
    step(1, r);
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { set_up_engine_in_game(r) };
    Ok(())
}

/// With the tables' Rust version and the cars through the game.
pub unsafe fn set_up_engine_in_game(r: &mut Regs) {
    set_up_engine(r, &mut |n, r| unsafe {
        if n == 0 {
            fill_engine_tables(r);
        } else {
            crate::calls::call(CARS_RVA, r);
        }
    });
}
