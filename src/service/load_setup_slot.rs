//! `service::load_setup_slot` (0x46EECC): the action of a slot on the "load setup" menu:
//! reads the slot's saved car setup and makes it the chosen setup.
//!
//! In:  nothing
//! Out: every register preserved
//! The slot is `dword_755244 * 30 + item - 1`, and one less from item 0x11 on (`dword_755244`
//! is read as the page of the slot list). If `read_setup_slot` fails, nothing more happens.
//! Otherwise: all repair choices are cleared; the file's six values go to the chosen tyre
//! (+0x30), the car's brake balance (+0x34), the chosen ride height (+0x38) and suspension
//! stiffness (+0x3C), the car's steering sensitivity (+0x40) and the chosen gear ratio
//! (+0x44); both flag tables are cleared; then, for gear ratio, tyre, ride height and
//! stiffness in that order, if the chosen value differs from the installed one, its edit page
//! is run as if confirmed, with its menu item highlighted (so the change is selected if the
//! service time allows); the highlighted item is put back, and `menu::go_back` leaves the
//! menu.
//! Refuses, before anything is written: no player, a slot past 99, a flag table without
//! its end mark.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::menu::go_back;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::pages_store::MENU_PAGES;
use crate::service::clear_repair_flags::clear_repair_flags;
use crate::service::clear_repair_marks::clear_repair_marks;
use crate::service::edit_gear_ratio::edit_gear_ratio;
use crate::service::edit_ride_height::edit_ride_height;
use crate::service::edit_suspension_stiffness::edit_suspension_stiffness;
use crate::service::edit_tyre::edit_tyre;
use crate::service::pages::{
    BRAKE_BALANCE, ChangePage, GEAR_RATIO, RIDE_HEIGHT, STEERING_SENSITIVITY, SUSPENSION_STIFFNESS,
    TYRE,
};
use crate::service::read_setup_slot::{SET_UPS, SLOT_COUNT, read_setup_slot};
use crate::service::state::{ITEM_COUNT, current_player, marks_ended, menu_item, repair_flag};

// ---- game data ----

/// The slot the highlighted menu item stands for.
/// A change page's editor (`confirm` its argument).
type Editor = fn(bool) -> Result<(), Refused>;

pub fn highlighted_slot() -> u32 {
    // Converted: `rc2k_core::service::setups::highlighted_slot`.
    rc2k_core::service::setups::highlighted_slot(MENU_PAGES.setup_slot_page.get(), menu_item())
}

// ---- Rust version ----

pub fn load_setup_slot() -> Result<(), Refused> {
    let slot = highlighted_slot();
    let Some(player) = current_player() else {
        return Err(BadInput("no current player object: the original writes through null").into());
    };
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table").into());
    }
    if repair_flag(ITEM_COUNT) != u32::MAX || !marks_ended() {
        return Err(BadInput("a flag table has lost its end mark").into());
    }

    if !read_setup_slot(slot)? {
        return Ok(());
    }
    clear_repair_flags()?;

    // Converted: `rc2k_core::service::setups` (`SetupValues`, `CONFIRM_ORDER`).
    use rc2k_core::service::setups::{CONFIRM_ORDER, EditPage, SetupValues};
    let v = SetupValues::from_file(&SET_UPS.file.read());
    let in_car = |field: rc2k_core::menu::player::SetupField, v: u32| {
        player.update(|q| *field.of(q) = v);
    };
    TYRE.change.chosen.set(v.tyre);
    in_car(BRAKE_BALANCE.in_car, v.brake_balance);
    RIDE_HEIGHT.change.chosen.set(v.ride_height);
    SUSPENSION_STIFFNESS
        .change
        .chosen
        .set(v.suspension_stiffness);
    in_car(STEERING_SENSITIVITY.in_car, v.steering_sensitivity);
    GEAR_RATIO.change.chosen.set(v.gear_ratio);

    // The original clears both tables again here, this time checking each entry for the
    // end mark before writing it. With the end marks in place that is the same thing.
    clear_repair_flags()?;
    clear_repair_marks()?;

    let highlighted = menu_item();
    for page in CONFIRM_ORDER {
        let (change, edit): (ChangePage, Editor) = match page {
            EditPage::GearRatio => (GEAR_RATIO, edit_gear_ratio),
            EditPage::Tyre => (TYRE, edit_tyre),
            EditPage::RideHeight => (RIDE_HEIGHT, edit_ride_height),
            _ => (SUSPENSION_STIFFNESS, edit_suspension_stiffness),
        };
        if change.change.chosen.get() != change.change.installed.get() {
            LOOP_FLAGS.item.set(change.item);
            edit(true)?;
        }
    }
    LOOP_FLAGS.item.set(highlighted);
    go_back::go_back();
    Ok(())
}

//
// Slots without a file only show that nothing happens. Each slot is tried with a service time
// that lets every change in, one that lets them in only by overrunning, and none.
