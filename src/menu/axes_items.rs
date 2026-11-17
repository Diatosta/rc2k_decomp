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

/// Whether the H shifter is switched on in the current player's profile for the controller
/// they have chosen (off with none chosen): the buttons page then lists the shifter's gears too.
pub fn shifter_on(menus: &Menus, input: &crate::input::owner::Input) -> bool {
    let current = menus.screens.event.current_player.get();
    let Ok(player) = crate::menu::player_settings::current(menus) else {
        return false;
    };
    crate::input::profiles::with_profile(input, current, player.get().controller, |p| p.shifter)
        .unwrap_or(false)
}

/// The controller page's shifter switch turned over, in the profile of the chosen controller
/// (nothing with no controller chosen).
pub fn toggle_shifter(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let menus = o.menus;
    let current = menus.screens.event.current_player.get();
    let controller = crate::menu::player_settings::current(menus)?
        .get()
        .controller;
    crate::input::profiles::with_profile(o.input, current, controller, |p| {
        p.shifter = !p.shifter;
    });
    Ok(())
}

/// The controller page's button pictures style one step, in the profile of the chosen controller
/// (nothing with no controller chosen).
pub fn step_icons(o: crate::owners::Owners<'static>, up: bool) -> Result<(), Refused> {
    let menus = o.menus;
    let current = menus.screens.event.current_player.get();
    let controller = crate::menu::player_settings::current(menus)?
        .get()
        .controller;
    crate::input::profiles::with_profile(o.input, current, controller, |p| {
        p.button_icons = p.button_icons.stepped(up);
    });
    Ok(())
}

/// How many pages of controls the buttons page has.
fn pages(shifter: bool) -> usize {
    Row::pages(shifter, ROWS_PER_PAGE)
}

/// The page shown, held to the pages there are (the switch may have been turned off since the
/// page was turned).
pub fn page_shown(controls: &ControlsScreen, shifter: bool) -> usize {
    (controls.page.get() as usize).min(pages(shifter) - 1)
}

/// The buttons page shows its next page of controls, the first after the last.
pub fn turn_controls_page(controls: &ControlsScreen, shifter: bool) {
    let next = (page_shown(controls, shifter) + 1) % pages(shifter);
    controls.page.set(next as u32);
}

/// The row of the buttons page that is item slot `slot` (0 ..= 7) on the page shown.
pub fn row_at(controls: &ControlsScreen, shifter: bool, slot: usize) -> Option<Row> {
    Row::on_page(shifter, ROWS_PER_PAGE, page_shown(controls, shifter), slot)
}

/// How many pages of controls the buttons page has (the last one the menu controls').
pub fn page_count(shifter: bool) -> usize {
    pages(shifter)
}

/// The text id of a row's label: the rows of the game's own controls, then the shifter's.
pub fn row_label(row: Row) -> u32 {
    use rc2k_core::strings::own::id;
    let place = row.place() as u32;
    let base = Row::LISTED.len() as u32;
    let gears_end = Row::ALL.len() as u32;
    if place < base {
        id::CONTROL_ROWS + place
    } else if place < gears_end {
        id::GEAR_ROWS + place - base
    } else {
        id::MENU_ROWS + place - gears_end
    }
}

/// The row at `slot` starts learning its control.
pub fn learn_row(o: crate::owners::Owners<'static>, slot: u8) {
    let controls = &o.menus.screens.controls;
    if let Some(row) = row_at(controls, shifter_on(o.menus, o.input), slot as usize) {
        crate::menu::learn_bindings::learn_binding(controls, o.system, o.input, row.control());
    }
}
