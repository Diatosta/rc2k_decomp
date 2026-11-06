use crate::errors::{BadInput, Refused};
use crate::menu::player::Player;

/// The current player's record, or a refusal.
pub fn current() -> Result<crate::menu::standings::PlayerCell, Refused> {
    let index: u32 = crate::menu::owner::menus()
        .screens
        .event
        .current_player
        .get();
    crate::menu::player::player(index).ok_or_else(|| BadInput("no current player").into())
}

/// One step of a setting: `field` picks it, `step` makes the next value.
pub fn change(field: fn(&mut Player) -> &mut u32, step: fn(u32) -> u32) -> Result<(), Refused> {
    current()?.update(|player| {
        let value = field(player);
        *value = step(*value);
    });
    Ok(())
}
