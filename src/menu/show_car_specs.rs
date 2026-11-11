//! `menu::show_car_specs` (0x46FD31): the car specifications screen for the kind of event
//! (`event.event_type`): 1 .. 3 the list the second car list, 4 the championship car list, each through
//! `menu::secret_car_list` (with cheat bit 1 the cheat car list instead), then
//! `menu::car_specs_display_loop` and, unless it was left (`captions.left_by_key` set),
//! `take_shown_car`; 5 the A8 championship car list and the screen alone. Another kind is the original's
//! fatal error 0x80713214, "No GameMode Defined".
//!
//! An item action (a reading: the main menu's "car specifications").

use crate::errors::{FatalError, Refused};
use crate::menu::lists_store::CarList;
use crate::menu::owner::Menus;
use crate::menu::screen_loop::{Entry, run_screen};

pub fn show_car_specs(menus: &Menus) -> Result<(), Refused> {
    // The logic is in `rc2k_core::menu::cars::specs_list`.
    use rc2k_core::menu::cars::{SpecsList, specs_list};
    let kind: u32 = menus.screens.event.event_type.get();
    let Some((list, choice)) = specs_list(kind) else {
        return Err(FatalError {
            code: 0x8071_3214,
            message: "No GameMode Defined",
        }
        .into());
    };
    let mut cars = match list {
        SpecsList::Events => CarList::Second,
        SpecsList::Kind4 => CarList::Championship,
        SpecsList::Kind5 => CarList::A8,
    };
    let screen = &crate::menu::car_specs_display_loop::SCREEN;
    if !choice {
        {
            run_screen(
                screen,
                &Entry::with_cars(crate::owners::boundary(), Some(cars)),
            )
        }?;
        return Ok(());
    }
    if let Some(cheat) = crate::menu::cheat_car_list::cheat_car_list(menus) {
        cars = cheat;
    }
    {
        run_screen(
            screen,
            &Entry::with_cars(crate::owners::boundary(), Some(cars)),
        )
    }?;
    if !menus.tables.captions.left_by_key.is_set() {
        crate::menu::take_shown_car::take_shown_car(menus, Some(cars))?;
    }
    Ok(())
}
