//! The championships' starters: the actions of the start menus' items that set a
//! championship up for one to four players and run it.
//!
//! The common shape: `event_type` is set (4, 5), `left_by_key` cleared, the carousel's last car
//! set to 0xF, `menu::reset_driver_condition`, `index`, `section` and `stages_done` cleared,
//! `player_count` set; for each player (`current_player`, edi its record) the repairs
//! (`service::repair_car`), the car choice (`menu::select_car_loop`, esi the championship's
//! car list; leaving it, `left_by_key` is set and ends it all) and the car taken
//! (`menu::take_shown_car`); the event mode is set; the championship
//! (`menu::run_championship` / `menu::run_a8_championship`); then for the first
//! championship `event_left`, `ask_leave` and `left_with_carry` are cleared, and for both
//! `event_over` is set. The one-player ones differ (see `Starter::first`).

use crate::errors::Refused;
use crate::menu::event_flow::*;
use crate::menu::lists_store::CarList;
use crate::menu::owner::Menus;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::session::PlainDword;
use rc2k_core::menu::leaving::Switch;
use rc2k_core::menu::starters;

const REPAIR_SET: &PlainDword = &crate::menu::owner::MENUS.unchecked().cars.table.last_car; // 0xF

pub struct Starter {
    /// The championship and the seats (see `rc2k_core::menu::starters`).
    pub kind: starters::Starter,
}

/// The game's variables and callees for `rc2k_core::menu::starters::start`, and the car
/// list the car choice was shown.
struct Game<'a> {
    menus: &'a crate::menu::owner::Menus,
    cars: Option<CarList>,
}

impl starters::Event for Game<'_> {
    type Error = Refused;

    fn get(&mut self, v: starters::Var) -> u32 {
        get(var(v))
    }

    fn set(&mut self, v: starters::Var, value: u32) {
        set(var(v), value);
    }

    fn flag(&mut self, s: Switch) -> bool {
        switch_cell(s).is_set()
    }

    fn set_flag(&mut self, s: Switch, on: bool) {
        switch_cell(s).set(on);
    }

    fn call(&mut self, c: starters::Call) -> Result<(), Refused> {
        use crate::menu as m;
        use starters::Call as C;
        match c {
            C::RepairCar => crate::service::repair_parts::repair_car()?,
            C::BeginEvent => m::begin_event::begin_event(self.menus)?,
            C::ResetDrivers => m::reset_drivers::reset_drivers(self.menus)?,
            C::ResetDriverCondition => {
                m::reset_driver_condition::reset_driver_condition(self.menus)?;
            }
            C::SelectCar(list) => {
                self.cars = Some(list);
                {
                    run_screen(
                        &m::select_car_loop::SCREEN,
                        &Entry::with_cars(self.menus, self.cars),
                    )
                }?;
            }
            C::TakeShownCar => m::take_shown_car::take_shown_car(self.menus, self.cars)?,
            C::HideAllInGameOptions => {
                m::hide_all_in_game_options::hide_all_in_game_options(self.menus)?;
            }
            C::RunA8Championship => m::run_a8_championship::run_a8_championship(self.menus)?,
            C::ShowInGameOptions => m::in_game_options_items::show_in_game_options(self.menus)?,
            C::RunChampionship => m::run_championship::run_championship(self.menus)?,
        }

        Ok(())
    }
}

fn var(v: starters::Var) -> &'static PlainDword {
    use starters::Var as V;
    match v {
        V::EventType => EVENT_TYPE,
        V::RepairSet => REPAIR_SET,
        V::PlayerCount => PLAYER_COUNT,
        V::Index => INDEX,
        V::Section => SECTION,
        V::StagesDone => crate::menu::run_championship::STAGES_DONE,
        V::Continuing => CONTINUING,
        V::Current => CURRENT,
        V::EventMode => EVENT_MODE,
    }
}

/// Starts the event of a starter.
pub fn start_event(menus: &Menus, s: &Starter) -> Result<(), Refused> {
    // The logic is in `rc2k_core::menu::starters::start`. The players' own settings come back
    // afterwards (`menu::own_settings`; not the original's).
    crate::menu::own_settings::around_championship(menus, || {
        starters::start(s.kind, &mut Game { menus, cars: None })
    })
}
