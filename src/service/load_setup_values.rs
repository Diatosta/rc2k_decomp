//! `service::load_setup_values` (0x47070C): copies three setup values of the current car into
//! the service area's edit variables, unless `dword_75AAE0` is 1.
//!
//! In:  nothing
//! Out: every register preserved
//! Each value goes into a pair of variables: car +0x10C into 0x75ACA0 and 0x75AC9C, +0x120
//! into 0x75AC94 and 0x75AC90, +0x118 into 0x75ACB8 and 0x75ACB4. A fourth pair is treated
//! differently: the function loads car +0x114 but then stores 0x75ACAC, which it does not
//! change, into 0x75ACA8. The names are a reading: what the three values are (the key handlers
//! step 0x75AC9C between 0 and 5) has not been worked out.
//! It is not called directly: its address is in the service area's list at 0x759436.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::service::state::current_player;

// ---- game data ----

const SKIP_RVA: usize = rva(0x75AAE0); // dword_75AAE0: 1 = leave the edit variables alone

// ---- Rust version ----

/// Refuses a missing player: the original reads through a null pointer.
pub unsafe fn load_setup_values() -> Result<(), BadInput> {
    unsafe {
        if game_read::<u32>(SKIP_RVA) == 1 {
            return Ok(());
        }
        let Some(player) = current_player() else {
            return Err(BadInput(
                "no current player object: the original reads through null",
            ));
        };
        // Converted: `rc2k_core::service::setups` (`Player::setup_edit_values`).
        let values = (player as *const rc2k_core::menu::player::Player)
            .read_unaligned()
            .setup_edit_values();
        for (value, (first, second)) in values.into_iter().zip([
            (0x75ACA0, 0x75AC9C),
            (0x75AC94, 0x75AC90),
            (0x75ACB8, 0x75ACB4),
        ]) {
            game_write(rva(first), value);
            game_write(rva(second), value);
        }
        // The original loads car +0x114 here and then stores this instead.
        let kept: u32 = game_read(rva(0x75ACAC));
        game_write(rva(0x75ACA8), kept);
        Ok(())
    }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { load_setup_values() }?;
    Ok(())
}
