//! `menu::drop_missing_controllers`: each player's controller number worked out from the
//! controller the player chose (`input::device_ids::give_chosen`): where it is in the table
//! now, none while it is not plugged in. The game itself only made a number past the
//! controller count none, so a player whose controller was not plugged in at start lost the
//! choice, and one whose controllers came in another order got another controller.

use crate::errors::BadInput;
use crate::input::owner::Input;
use crate::menu::owner::Menus;

pub fn drop_missing_controllers(menus: &Menus, input: &Input) -> Result<(), BadInput> {
    crate::input::device_ids::give_chosen(menus, input);
    Ok(())
}
