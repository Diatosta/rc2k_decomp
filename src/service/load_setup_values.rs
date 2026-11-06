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
use crate::menu::owner::menus;
use crate::service::state::current_player_of;

// ---- game data ----

// ---- Rust version ----

/// Refuses a missing player: the original reads through a null pointer.
pub fn load_setup_values() -> Result<(), BadInput> {
    let screens = &menus().screens;
    {
        if screens.state.menus.skip_edit_values.get() == 1 {
            return Ok(());
        }
        let Some(player) = current_player_of(screens) else {
            return Err(BadInput(
                "no current player object: the original reads through null",
            ));
        };
        // Converted: `rc2k_core::service::setups` (`Player::setup_edit_values`).
        let values = player.get().setup_edit_values();
        let setups = &screens.state.driver_stats.setups;
        for (value, item) in values.into_iter().zip([&setups[1], &setups[0], &setups[3]]) {
            item.installed.set(value);
            item.chosen.set(value);
        }
        // The original loads car +0x114 here and then stores this instead.
        let kept: u32 = screens.state.driver_stats.setups[2].installed.get();
        screens.state.driver_stats.setups[2].chosen.set(kept);
        Ok(())
    }
}
