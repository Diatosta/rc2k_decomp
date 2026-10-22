//! Calls of the game's functions by address from Rust (conversion, 2026-10-22): where a
//! Rust version named a callee by its address (`call_game(r, game_addr(..))`), it calls
//! `call` with the same address, which runs the callee's Rust version
//! (`menu::table_routes`, `routes`) without its hook, by the rules of
//! `menu::table_calls`: in shadow mode the game's (its hook compares), a refusal logged
//! once and the game's run, a fatal error raised; an address with no Rust version goes to
//! the game, as before.

use crate::framework::{Regs, game_addr};

/// Runs the game's function at `at` (an RVA, as `game_addr` takes it) with `r`.
pub fn call(at: usize, r: &mut Regs) {
    crate::menu::table_calls::call_any(game_addr(at) as u32, r);
}
