//! The car's parts repaired whole: for each of five parts a run of the
//! current player's car's dwords (as `service::apply_repairs` zeroes them) and its damage
//! dword zeroed, and `service::repair_car`, which does all of them and more. Converted:
//! `rc2k_core::service::car`.

use crate::errors::BadInput;
use crate::service::state::current_player;
use rc2k_core::menu::player::Player;

pub fn player() -> Result<crate::menu::standings::PlayerCell<'static>, BadInput> {
    current_player().ok_or(BadInput(
        "no current player object: the original writes through null",
    ))
}

/// The current player's record changed by `f` (read, changed, written back whole).
pub fn with_player(f: impl FnOnce(&mut Player)) -> Result<(), BadInput> {
    player()?.update(f);
    Ok(())
}

/// `service::repair_car`: the current car as new: the damage dwords, the body, the tyres'
/// wear, the gearbox, the brakes, the lights, the suspension, the tyres' wear again, the
/// flags -1 and +0x24C's eight dwords.
pub fn repair_car() -> Result<(), BadInput> {
    // Converted: `rc2k_core::service::car` (`Player::repair_car`).
    with_player(Player::repair_car)
}
