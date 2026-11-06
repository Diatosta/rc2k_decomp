//! `service::handle_edit_keys` (0x46B890): while an item of the service area is being edited
//! (`dword_75AA9C == 1`), passes the keys to the edit page in `dword_75AAA0`: 1 gear ratio,
//! 2 tyre, 3 steering sensitivity, 4 brake balance, 5 ride height, 6 suspension stiffness.
//!
//! In:  nothing
//! Out: every register preserved
//! Any other page number is fatal error 0x80709A9A, "Incorrect Locked Box", in the original.
//! It is not called directly: its address is in the service area's list at 0x758D6E.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::FatalError;
use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::service::edit_brake_balance::edit_brake_balance;
use crate::service::edit_gear_ratio::edit_gear_ratio;
use crate::service::edit_ride_height::edit_ride_height;
use crate::service::edit_steering_sensitivity::edit_steering_sensitivity;
use crate::service::edit_suspension_stiffness::edit_suspension_stiffness;
use crate::service::edit_tyre::edit_tyre;
use crate::service::state::edit_mode;

// ---- Rust version ----

pub fn handle_edit_keys() -> Result<(), Refused> {
    if !edit_mode() {
        return Ok(());
    }
    // Converted: `rc2k_core::service::setups::EditPage`.
    use rc2k_core::service::setups::EditPage;
    match EditPage::from_number(menus().screens.state.service_ui.box_page.get()) {
        Some(EditPage::GearRatio) => edit_gear_ratio(false),
        Some(EditPage::Tyre) => edit_tyre(false),
        Some(EditPage::SteeringSensitivity) => edit_steering_sensitivity(),
        Some(EditPage::BrakeBalance) => edit_brake_balance(),
        Some(EditPage::RideHeight) => edit_ride_height(false),
        Some(EditPage::SuspensionStiffness) => edit_suspension_stiffness(false),
        None => Err(FatalError {
            code: 0x8070_9A9A,
            message: "Incorrect Locked Box",
        }
        .into()),
    }
}
