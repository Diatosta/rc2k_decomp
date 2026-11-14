//! Copies three setup values of the current car into the service area's edit variables, unless
//! `skip_edit_values` is 1. Each value goes into a pair of variables (installed and chosen);
//! the fourth pair's chosen value becomes its installed one, the car's value not read. What the
//! values are has not been worked out: the names are a reading.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::service::owner::service;
use crate::service::state::current_player;

/// Refuses a missing player.
pub fn load_setup_values(menus: &Menus) -> Result<(), BadInput> {
    let screens = &menus.screens;
    {
        if screens.state.menus.skip_edit_values.is_set() {
            return Ok(());
        }
        let Some(player) = current_player(menus) else {
            return Err(BadInput(
                "no current player object: the original reads through null",
            ));
        };
        let values = player.get().setup_edit_values();
        let setups = &service().setups;
        for (value, item) in values.into_iter().zip([&setups[1], &setups[0], &setups[3]]) {
            item.installed.set(value);
            item.chosen.set(value);
        }
        // The fourth pair: its chosen value becomes its installed one (kept as is: the car's
        // fourth value is not loaded).
        let kept: u32 = service().setups[2].installed.get();
        service().setups[2].chosen.set(kept);
        Ok(())
    }
}
