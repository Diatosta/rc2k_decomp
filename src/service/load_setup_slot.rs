//! `service::load_setup_slot` (0x46EECC): the action of a slot on the "load setup" menu:
//! reads the slot's saved car setup and makes it the chosen setup.
//!
//! The slot is `setup_slot_page * 30 + item - 1`, and one less from item 0x11 on (`setup_slot_page` is
//! read as the page of the slot list). If `read_setup_slot` fails, nothing more happens. Otherwise:
//! all repair choices are cleared; the file's six values go to the chosen tyre (+0x30), the car's
//! brake balance (+0x34), the chosen ride height (+0x38) and suspension stiffness (+0x3C), the
//! car's steering sensitivity (+0x40) and the chosen gear ratio (+0x44); both flag tables are
//! cleared; then, for gear ratio, tyre, ride height and stiffness in that order, if the chosen
//! value differs from the installed one, its edit page is run as if confirmed, with its menu item
//! highlighted (so the change is selected if the service time allows); the highlighted item is put
//! back, and `menu::go_back` leaves the menu. Refuses, before anything is written: no player, a
//! slot past 99, a flag table without its end mark.

use crate::errors::{BadInput, Refused};
use crate::menu::go_back;
use crate::menu::owner::Menus;
use crate::service::clear_repair_flags::clear_repair_flags;
use crate::service::clear_repair_marks::clear_repair_marks;
use crate::service::edit_gear_ratio::edit_gear_ratio;
use crate::service::edit_ride_height::edit_ride_height;
use crate::service::edit_suspension_stiffness::edit_suspension_stiffness;
use crate::service::edit_tyre::edit_tyre;
use crate::service::owner::Service;
use crate::service::pages::{
    BRAKE_BALANCE, ChangePage, GEAR_RATIO, RIDE_HEIGHT, STEERING_SENSITIVITY, SUSPENSION_STIFFNESS,
    TYRE,
};
use crate::service::read_setup_slot::{SLOT_COUNT, read_setup_slot};
use crate::service::state::{ITEM_COUNT, current_player, marks_ended, repair_flag_in};
use crate::system::owner::System;

// ---- game data ----

/// The slot the highlighted menu item stands for.
/// A change page's editor (`confirm` its argument).
type Editor = fn(&Menus, &System, bool) -> Result<(), Refused>;

pub fn highlighted_slot(menus: &Menus) -> u32 {
    // Converted: `rc2k_core::service::setups::highlighted_slot`.
    rc2k_core::service::setups::highlighted_slot(
        menus.tables.pages.setup_slot_page.get(),
        menus.screens.loop_flags.item.get(),
    )
}

// ---- Rust version ----

pub fn load_setup_slot(menus: &Menus, service: &Service, system: &System) -> Result<(), Refused> {
    let (screens, tables) = (&menus.screens, &menus.tables);
    let slot = highlighted_slot(menus);
    let Some(player) = current_player(menus) else {
        return Err(BadInput("no current player object: the original writes through null").into());
    };
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table").into());
    }
    if repair_flag_in(tables, ITEM_COUNT) != u32::MAX || !marks_ended(tables) {
        return Err(BadInput("a flag table has lost its end mark").into());
    }

    if !read_setup_slot(service, slot)? {
        return Ok(());
    }
    clear_repair_flags(menus)?;

    // Converted: `rc2k_core::service::setups` (`SetupValues`, `CONFIRM_ORDER`).
    use rc2k_core::service::setups::{CONFIRM_ORDER, EditPage, SetupValues};
    let v = SetupValues::from_file(&service.set_ups.file.read());
    let in_car = |field: rc2k_core::menu::player::SetupField, v: u32| {
        player.update(|q| *field.of(q) = v);
    };
    TYRE.change.chosen(menus).set(v.tyre);
    in_car(BRAKE_BALANCE.in_car, v.brake_balance);
    RIDE_HEIGHT.change.chosen(menus).set(v.ride_height);
    SUSPENSION_STIFFNESS
        .change
        .chosen(menus)
        .set(v.suspension_stiffness);
    in_car(STEERING_SENSITIVITY.in_car, v.steering_sensitivity);
    GEAR_RATIO.change.chosen(menus).set(v.gear_ratio);

    // The original clears both tables again here, this time checking each entry for the
    // end mark before writing it. With the end marks in place that is the same thing.
    clear_repair_flags(menus)?;
    clear_repair_marks(tables)?;

    let highlighted = screens.loop_flags.item.get();
    for page in CONFIRM_ORDER {
        let (change, edit): (ChangePage, Editor) = match page {
            EditPage::GearRatio => (GEAR_RATIO, edit_gear_ratio),
            EditPage::Tyre => (TYRE, edit_tyre),
            EditPage::RideHeight => (RIDE_HEIGHT, edit_ride_height),
            _ => (SUSPENSION_STIFFNESS, edit_suspension_stiffness),
        };
        if change.change.chosen(menus).get() != change.change.installed(menus).get() {
            screens.loop_flags.item.set(change.item);
            edit(menus, system, true)?;
        }
    }
    screens.loop_flags.item.set(highlighted);
    go_back::go_back(menus);
    Ok(())
}

//
// Slots without a file only show that nothing happens. Each slot is tried with a service time
// that lets every change in, one that lets them in only by overrunning, and none.
