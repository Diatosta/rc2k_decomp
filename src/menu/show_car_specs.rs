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
use crate::framework::{game_addr, game_read, rva};
use crate::menu::screen_loop::{Carried, run_screen_carried};

const KIND_RVA: usize = rva(0x755954); // dword_755954
const LEFT_RVA: usize = rva(0x758806); // dword_758806

/// The Rust version. `c` the carried values the item gets. (The original leaves the
/// registers its callees leave, esi the list; an item's caller reads none.)
pub unsafe fn show_car_specs(c: &Carried) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::cars::specs_list`.
    use rc2k_core::menu::cars::{SpecsList, specs_list};
    let mut r = *c;
    let kind: u32 = unsafe { game_read(KIND_RVA) };
    let Some((list, choice)) = specs_list(kind) else {
        return Err(FatalError {
            code: 0x8071_3214,
            message: "No GameMode Defined",
        }
        .into());
    };
    r.cars = game_addr(rva(match list {
        SpecsList::Events => 0x75A042,
        SpecsList::Kind4 => 0x759CAE,
        SpecsList::Kind5 => 0x759D16,
    })) as u32;
    if !choice {
        unsafe { run_screen_carried(&crate::menu::car_specs_47af4c_loop::SCREEN, &mut r) }?;
        return Ok(());
    }
    crate::menu::cheat_car_list::carry_on(crate::menu::cheat_car_list::cheat_car_list(), &mut r);
    unsafe { run_screen_carried(&crate::menu::car_specs_47af4c_loop::SCREEN, &mut r) }?;
    if unsafe { game_read::<u32>(LEFT_RVA) } != 1 {
        unsafe { crate::menu::take_shown_car::take_shown_car(r.cars) }?;
    }
    Ok(())
}
