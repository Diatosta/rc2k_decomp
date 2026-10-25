//! `menu::show_car_specs` (0x46FD31): the car specifications screen for the kind of event
//! (`dword_755954`): 1 .. 3 the list `off_75A042`, 4 `off_759CAE`, each through
//! `menu::secret_car_list` (with cheat bit 1 `off_75A412` instead), then
//! `menu::car_specs_47af4c_loop` and, unless it was left (`dword_758806` 1),
//! `take_shown_car`; 5 `off_759D16` and the screen alone. Another kind is the original's
//! fatal error 0x80713214, "No GameMode Defined".
//!
//! In:  nothing
//! Out: the registers as its callees leave them (not `pusha`'d); esi the list
//! An item action (a reading: the main menu's "car specifications").

use crate::errors::{FatalError, Refused};
use crate::menu::captions_store::CAPTIONS;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::screen_loop::{Carried, run_screen_carried};

/// The Rust version. `c` the carried values the item gets. (The original leaves the
/// registers its callees leave, esi the list; an item's caller reads none.)
pub unsafe fn show_car_specs(c: &Carried) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::cars::specs_list`.
    use rc2k_core::menu::cars::{SpecsList, specs_list};
    let mut r = *c;
    let kind: u32 = { EVENT_STATE.event_type.get() };
    let Some((list, choice)) = specs_list(kind) else {
        return Err(FatalError {
            code: 0x8071_3214,
            message: "No GameMode Defined",
        }
        .into());
    };
    r.cars = match list {
        SpecsList::Events => MENU_LISTS.cars_second[0].addr(),
        SpecsList::Kind4 => MENU_LISTS.cars_championship[0].addr(),
        SpecsList::Kind5 => MENU_LISTS.cars_a8[0].addr(),
    } as u32;
    if !choice {
        unsafe { run_screen_carried(&crate::menu::car_specs_47af4c_loop::SCREEN, &mut r) }?;
        return Ok(());
    }
    crate::menu::cheat_car_list::carry_on(crate::menu::cheat_car_list::cheat_car_list(), &mut r);
    unsafe { run_screen_carried(&crate::menu::car_specs_47af4c_loop::SCREEN, &mut r) }?;
    if { CAPTIONS.screen_left.get() } != 1 {
        unsafe { crate::menu::take_shown_car::take_shown_car(r.cars) }?;
    }
    Ok(())
}
