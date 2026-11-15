//! The action of a slot on the "load setup" menu: reads the slot's saved car setup and makes it
//! the chosen setup.
//!
//! The slot is `setup_slot_page * 30 + item - 1`, and one less from item 0x11 on. If
//! `read_setup_slot` fails, nothing more happens. Otherwise: all repair choices are cleared;
//! the file's six values go to the chosen tyre, the car's brake balance, the chosen ride height
//! and suspension stiffness, the car's steering sensitivity and the chosen gear ratio; both
//! flag tables are cleared; then, for gear ratio, tyre, ride height and stiffness in that
//! order, if the chosen value differs from the installed one, its edit page is run as if
//! confirmed, with its menu item highlighted (so the change is selected if the service time
//! allows); the highlighted item is put back, and `menu::go_back` leaves the menu. Refuses,
//! before anything is written: no player, a slot past 99, a flag table without its end mark.

use crate::errors::{BadInput, Refused};
use crate::menu::go_back;
use crate::menu::loop_store::LoopFlags;
use crate::menu::owner::Menus;
use crate::menu::pages_store::MenuPages;
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

/// A change page's editor (`confirm` its argument).
type Editor = fn(&Menus, &System, &crate::input::owner::Input, bool) -> Result<(), Refused>;

/// The slot the highlighted menu item stands for.
pub fn highlighted_slot(pages: &MenuPages, loop_flags: &LoopFlags) -> u32 {
    rc2k_core::service::setups::highlighted_slot(pages.setup_slot_page.get(), loop_flags.item.get())
}

pub fn load_setup_slot(
    menus: &Menus,
    service: &Service,
    system: &System,
    input: &crate::input::owner::Input,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    let slot = highlighted_slot(&menus.tables.pages, &menus.screens.loop_flags);
    let Some(player) = current_player(menus) else {
        return Err(BadInput("no current player object: the original writes through null").into());
    };
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table").into());
    }
    if repair_flag_in(ITEM_COUNT) != u32::MAX || !marks_ended() {
        return Err(BadInput("a flag table has lost its end mark").into());
    }

    if !read_setup_slot(service, slot)? {
        return Ok(());
    }
    clear_repair_flags()?;

    use rc2k_core::service::setups::{CONFIRM_ORDER, EditPage, SetupValues};
    let v = SetupValues::from_file(&service.set_ups.file.read());
    let in_car = |field: rc2k_core::menu::player::SetupField, v: u32| {
        player.update(|q| *field.of(q) = v);
    };
    TYRE.change.chosen().set(v.tyre);
    in_car(BRAKE_BALANCE.in_car, v.brake_balance);
    RIDE_HEIGHT.change.chosen().set(v.ride_height);
    SUSPENSION_STIFFNESS
        .change
        .chosen()
        .set(v.suspension_stiffness);
    in_car(STEERING_SENSITIVITY.in_car, v.steering_sensitivity);
    GEAR_RATIO.change.chosen().set(v.gear_ratio);

    // Both tables are cleared again; with the end marks in place the flags' second clear
    // changes nothing.
    clear_repair_flags()?;
    clear_repair_marks()?;

    let highlighted = screens.loop_flags.item.get();
    for page in CONFIRM_ORDER {
        let (change, edit): (ChangePage, Editor) = match page {
            EditPage::GearRatio => (GEAR_RATIO, edit_gear_ratio),
            EditPage::Tyre => (TYRE, edit_tyre),
            EditPage::RideHeight => (RIDE_HEIGHT, edit_ride_height),
            _ => (SUSPENSION_STIFFNESS, edit_suspension_stiffness),
        };
        if change.change.chosen().get() != change.change.installed().get() {
            screens.loop_flags.item.set(change.item);
            edit(menus, system, input, true)?;
        }
    }
    screens.loop_flags.item.set(highlighted);
    go_back::go_back(menus);
    Ok(())
}

// Slots without a file only show that nothing happens. Each slot is tried with a service time
// that lets every change in, one that lets them in only by overrunning, and none.
