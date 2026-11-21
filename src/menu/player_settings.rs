use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::player::Player;

/// The current player's record, or a refusal.
pub fn current(menus: &Menus) -> Result<crate::menu::standings::PlayerCell<'_>, Refused> {
    let index: u32 = menus.screens.event.read().current_player;
    crate::menu::player::player(menus, index).ok_or_else(|| BadInput("no current player").into())
}

/// One step of a setting: `field` picks it, `step` makes the next value.
pub fn change(
    menus: &Menus,
    field: fn(&mut Player) -> &mut u32,
    step: fn(u32) -> u32,
) -> Result<(), Refused> {
    current(menus)?.update(|player| {
        let value = field(player);
        *value = step(*value);
    });
    Ok(())
}
