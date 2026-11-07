//! The championships' starters (IDA, 2026-10-19): the actions of the start menus' items that set a
//! championship up for one to four players and run it.
//!
//! The common shape: `dword_755954` the event type (4, 5), `dword_758806` 0,
//! `dword_75649C` 0xF, `menu::reset_driver_condition`, `index`, `section` and
//! `dword_755950` 0, `dword_755984` the players; for each player (`dword_75598C`, edi its
//! record) the repairs (`service::repair_car`), the car choice (`menu::select_car_loop`,
//! esi the championship's car list; leaving it, `dword_758806` 1, ends it all) and the car
//! taken (`menu::take_shown_car`); `dword_755988` the event mode; the championship
//! (`menu::run_championship` / `menu::run_a8_championship`); then for the first
//! championship `dword_75555D`, `dword_755569` and `dword_755565` 0, and for both
//! `dword_755980` 1. The one-player ones differ (see `Starter::first`).

use crate::errors::Refused;
use crate::menu::event_flow::*;
use crate::menu::lists_store::CarList;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::session::Dword;
use rc2k_core::menu::starters;

const REPAIR_SET: &Dword = &crate::menu::owner::MENUS.cars.table.last_car; // dword_75649C: 0xF

pub struct Starter {
    /// The championship and the seats (see `rc2k_core::menu::starters`).
    pub kind: starters::Starter,
}

/// The game's variables and callees for `rc2k_core::menu::starters::start`, and the car
/// list the car choice was shown.
struct Game {
    menus: &'static crate::menu::owner::Menus,
    cars: Option<CarList>,
}

impl starters::Event for Game {
    type Error = Refused;

    fn get(&mut self, v: starters::Var) -> u32 {
        get(var(v))
    }

    fn set(&mut self, v: starters::Var, value: u32) {
        set(var(v), value);
    }

    fn call(&mut self, c: starters::Call) -> Result<(), Refused> {
        use crate::menu as m;
        use starters::Call as C;
        match c {
            C::RepairCar => crate::service::repair_parts::repair_car()?,
            C::BeginEvent => m::begin_event::begin_event(self.menus)?,
            C::ResetDrivers => m::reset_drivers::reset_drivers()?,
            C::ResetDriverCondition => m::reset_driver_condition::reset_driver_condition()?,
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
            C::ResetCars750724 => m::reset_cars_750724::reset_cars_750724(self.menus)?,
            C::RunA8Championship => m::run_a8_championship::run_a8_championship()?,
            C::ShowInGameOptions => m::in_game_options_items::show_in_game_options(self.menus)?,
            C::RunChampionship => m::run_championship::run_championship()?,
        }

        Ok(())
    }
}

fn var(v: starters::Var) -> &'static Dword {
    use starters::Var as V;
    match v {
        V::EventType => EVENT_TYPE,
        V::CarLeft => CAR_LEFT,
        V::RepairSet => REPAIR_SET,
        V::PlayerCount => PLAYER_COUNT,
        V::Index => INDEX,
        V::Section => SECTION,
        V::StagesDone => crate::menu::run_championship::STAGES_DONE,
        V::Continuing => CONTINUING,
        V::Current => CURRENT,
        V::EventMode => EVENT_MODE,
        V::Leave => LEAVE,
        V::Ended569 => ENDED_569,
        V::Ended565 => ENDED_565,
        V::Done => DONE,
    }
}

/// The Rust version of a starter.
pub fn start_event(s: &Starter) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::starters::start`.
    starters::start(
        s.kind,
        &mut Game {
            menus: crate::menu::owner::menus(),
            cars: None,
        },
    )
}
