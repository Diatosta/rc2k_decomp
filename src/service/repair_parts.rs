//! The car's parts repaired whole: for each of five parts a run of the current player's car's
//! dwords (as `service::apply_repairs` zeroes them) and its damage dword zeroed, and
//! `service::repair_car`, which does all of them and more.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::service::state::current_player;
use rc2k_core::menu::player::Player;

pub fn player(menus: &Menus) -> Result<crate::menu::standings::PlayerCell<'_>, BadInput> {
    current_player(menus).ok_or(BadInput(
        "no current player object: the original writes through null",
    ))
}

/// The current player's record changed by `f` (read, changed, written back whole).
pub fn with_player(menus: &Menus, f: impl FnOnce(&mut Player)) -> Result<(), BadInput> {
    player(menus)?.update(f);
    Ok(())
}

/// The current car as new: the damage dwords, the body, the tyres' wear, the gearbox, the
/// brakes, the lights, the suspension, the tyres' wear again, the flags -1 and eight more
/// dwords.
pub fn repair_car_in(menus: &Menus) -> Result<(), BadInput> {
    with_player(menus, Player::repair_car)
}
