//! The controller pages' item actions of the axes pages and of the buttons page's two pages of
//! controls: the wheel and the axis settings changed by a step, the axis picked, the page of
//! controls turned, and a row's control learnt. All of them are about the current player's profile for the
//! controller they have chosen (`input::profiles`).

use crate::errors::{BadInput, Refused};
use crate::menu::item_at::item_at;
use crate::menu::owner::Menus;
use crate::screens::controls_store::ControlsScreen;
use rc2k_core::input::axes_page::{Column, rows, step_rotation};
use rc2k_core::input::profile::Row;
use rc2k_core::menu::items::{HIDDEN, Look};
use rc2k_core::menu::navigation::CAN_BE_CHOSEN;
use rc2k_core::screens::menus::NOTHING_EITHER;

/// How many rows of controls one page of the buttons page lists.
pub const ROWS_PER_PAGE: usize = 8;

/// The look of a choosable row with `string`.
pub fn shown_look(string: u32) -> Look {
    Look {
        string,
        state: CAN_BE_CHOSEN,
        behind: NOTHING_EITHER,
    }
}

/// Item `number` (from 1) of the current menu given `look`; a hidden one is passed over.
pub fn set_look(menus: &Menus, number: u32, look: Look, hidden: bool) -> Result<(), BadInput> {
    let item = item_at(menus, number)?;
    let look = if hidden { HIDDEN } else { look };
    item.string.set(look.string);
    item.skip_mark.set(look.state);
    item.behind.set(look.behind);
    Ok(())
}

/// The wheel switch (item 8, either way) or the wheel's full rotation (item 9) of the axes list
/// one step.
pub fn step(o: crate::owners::Owners<'static>, up: bool) -> Result<(), Refused> {
    let menus = o.menus;
    let screens = &menus.screens;
    let item: u32 = screens.loop_flags.item.get();
    let current = screens.event.current_player.get();
    let controller = crate::menu::player_settings::current(menus)?
        .get()
        .controller;
    crate::input::profiles::with_profile(o.input, current, controller, |profile| match item {
        8 => profile.wheel = !profile.wheel,
        9 if profile.wheel => {
            profile.full_rotation_degrees = step_rotation(profile.full_rotation_degrees, up);
        }
        _ => {}
    });
    Ok(())
}

/// The axis page's highlighted row one step: the axis picked (item 2, round the axes the
/// controller has), or the axis's dead zone, sensitivity, range or invert (items 3 .. 6).
pub fn page_step(o: crate::owners::Owners<'static>, up: bool) -> Result<(), Refused> {
    let menus = o.menus;
    let screens = &menus.screens;
    let item: u32 = screens.loop_flags.item.get();
    let current = screens.event.current_player.get();
    let controller = crate::menu::player_settings::current(menus)?
        .get()
        .controller;
    let axes = crate::input::profiles::controller_kind(o.input, controller)
        .map_or(1, |(_, axes)| rows(axes).max(1) as u32);
    let axis = screens.controls.axis.get().min(axes - 1);
    if item == 2 {
        let next = if up { axis + 1 } else { axis + axes - 1 } % axes;
        screens.controls.axis.set(next);
        return Ok(());
    }
    let Some(&setting) = Column::ALL.get(item as usize - 3) else {
        return Ok(());
    };
    crate::input::profiles::with_profile(o.input, current, controller, |profile| {
        rc2k_core::input::axes_page::step(profile.settings_mut(axis as u8), setting, up);
    });
    Ok(())
}

/// The buttons page shows its other page of controls.
pub fn turn_controls_page(controls: &ControlsScreen) {
    controls.page.set(1 - controls.page.get().min(1));
}

/// The row of the buttons page that is item slot `slot` (0 ..= 7) on the page shown.
pub fn row_at(controls: &ControlsScreen, slot: usize) -> Option<Row> {
    Row::LISTED
        .get(controls.page.get() as usize * ROWS_PER_PAGE + slot)
        .copied()
}

/// The row at `slot` starts learning its control.
pub fn learn_row(o: crate::owners::Owners<'static>, slot: u8) {
    if let Some(row) = row_at(&o.menus.screens.controls, slot as usize) {
        crate::menu::learn_bindings::learn_binding(
            &o.menus.screens.controls,
            o.system,
            o.input,
            row.control(),
        );
    }
}
