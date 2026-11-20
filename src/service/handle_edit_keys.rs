//! While an item of the service area is being edited (`box_shown == 1`), passes the keys to the
//! edit page in `box_page`: 1 gear ratio, 2 tyre, 3 steering sensitivity, 4 brake balance,
//! 5 ride height, 6 suspension stiffness. Any other page number is fatal error 0x80709A9A,
//! "Incorrect Locked Box".

use crate::errors::FatalError;
use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::service::edit_brake_balance::edit_brake_balance;
use crate::service::edit_gear_ratio::edit_gear_ratio;
use crate::service::edit_ride_height::edit_ride_height;
use crate::service::edit_steering_sensitivity::edit_steering_sensitivity;
use crate::service::edit_suspension_stiffness::edit_suspension_stiffness;
use crate::service::edit_tyre::edit_tyre;
use crate::service::state::edit_mode;
use crate::system::owner::System;

pub fn handle_edit_keys(
    menus: &Menus,
    system: &System,
    input: &crate::input::owner::Input,
) -> Result<(), Refused> {
    if !edit_mode(menus) {
        return Ok(());
    }
    use rc2k_core::service::setups::EditPage;
    match EditPage::from_number(menus.screens.state.read().service_ui.box_page) {
        Some(EditPage::GearRatio) => edit_gear_ratio(menus, system, input, false),
        Some(EditPage::Tyre) => edit_tyre(menus, system, input, false),
        Some(EditPage::SteeringSensitivity) => edit_steering_sensitivity(menus, system, input),
        Some(EditPage::BrakeBalance) => edit_brake_balance(menus, system, input),
        Some(EditPage::RideHeight) => edit_ride_height(menus, system, input, false),
        Some(EditPage::SuspensionStiffness) => {
            edit_suspension_stiffness(menus, system, input, false)
        }
        None => Err(FatalError {
            code: 0x8070_9A9A,
            message: "Incorrect Locked Box",
        }
        .into()),
    }
}
