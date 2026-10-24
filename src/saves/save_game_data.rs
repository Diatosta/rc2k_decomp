//! `saves::save_game_data` (0x411499): writes what the game keeps between runs.
//!
//! In:  nothing
//! Out: every register preserved; the flags as `save_best_times` leaves them
//! `saves::save_options`, then `saves::save_best_times`.
//! Callers: the main loop after the front end is left, and `sub_48AD73`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_addr, rva};

// ---- Rust version ----

/// The two parts, each saying whether it wrote its file. Returns what the second said.
pub unsafe fn save_game_data(
    save_options: &mut dyn FnMut() -> Result<bool, Refused>,
    save_best_times: &mut dyn FnMut() -> Result<bool, Refused>,
) -> Result<bool, Refused> {
    save_options()?;
    save_best_times()
}

/// The two parts on the game's files: whether the best times were written. The original's
/// flags (carry: not written) are read by neither of its callers (`system::main_loop`,
/// `menu::arcade_level_end`; register liveness, 2026-10-23).
pub fn save_in_game() -> Result<bool, Refused> {
    unsafe {
        save_game_data(
            &mut || {
                crate::saves::save_options::save_options(
                    game_addr(rva(0x5295D8)) as *const u8, // "var\ral.dat"
                )
            },
            &mut || crate::saves::save_best_times::in_game(),
        )
    }
}
