//! The championships' starters: the actions of the start menus' items that set a championship
//! up for one to four players and run it.
//!
//! The common shape: `event_type` is set (4, 5), `left_by_key` cleared, the carousel's last car
//! set to 0xF, `menu::reset_driver_condition`, `index`, `section` and `stages_done` cleared,
//! `player_count` set; for each player (`current_player`) the repairs (`service::repair_car`),
//! the car choice (`menu::select_car_loop` with the championship's car list; leaving it,
//! `left_by_key` is set and ends it all) and the car taken (`menu::take_shown_car`); the event
//! mode is set; the championship (`menu::run_championship` / `menu::run_a8_championship`);
//! then for the first championship `event_left`, `ask_leave` and `left_with_carry` are
//! cleared, and for both `event_over` is set. The one-player ones differ (see `Starter::first`).

use crate::errors::Refused;
use crate::menu::event_flow::*;
use crate::menu::lists_store::CarList;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::session::PlainDword;
use rc2k_core::menu::leaving::Switch;
use rc2k_core::menu::starters;

pub struct Starter {
    /// The championship and the seats (see `rc2k_core::menu::starters`).
    pub kind: starters::Starter,
}

/// The game's variables and callees for `rc2k_core::menu::starters::start`, and the car
/// list the car choice was shown.
struct Game<'a> {
    menus: &'a crate::menu::owner::Menus,
    owners: crate::owners::Owners<'static>,
    cars: Option<CarList>,
}

impl starters::Event for Game<'_> {
    type Error = Refused;

    fn get(&mut self, v: starters::Var) -> u32 {
        get(var(self.menus, v))
    }

    fn set(&mut self, v: starters::Var, value: u32) {
        set(var(self.menus, v), value);
    }

    fn flag(&mut self, s: Switch) -> bool {
        switch_cell(self.owners, s).is_set()
    }

    fn set_flag(&mut self, s: Switch, on: bool) {
        switch_cell(self.owners, s).set(on);
    }

    fn call(&mut self, c: starters::Call) -> Result<(), Refused> {
        use crate::menu as m;
        use starters::Call as C;
        match c {
            C::RepairCar => crate::service::repair_parts::repair_car_in(self.menus)?,
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
                        &Entry::with_cars(self.owners, self.cars),
                    )
                }?;
            }
            C::TakeShownCar => m::take_shown_car::take_shown_car(self.menus, self.cars)?,
            C::HideAllInGameOptions => {
                m::hide_all_in_game_options::hide_all_in_game_options(self.menus)?;
            }
            C::RunA8Championship => m::run_a8_championship::run_a8_championship_in(self.owners)?,
            C::ShowInGameOptions => m::in_game_options_items::show_in_game_options(self.menus)?,
            C::RunChampionship => m::run_championship::run_championship_in(self.owners)?,
        }

        Ok(())
    }
}

fn var(menus: &crate::menu::owner::Menus, v: starters::Var) -> &PlainDword {
    use starters::Var as V;
    let event = &menus.screens.event;
    match v {
        V::EventType => &event.event_type,
        V::RepairSet => &menus.cars.table.last_car, // 0xF
        V::PlayerCount => &event.player_count,
        V::Index => &event.index,
        V::Section => &event.section,
        V::StagesDone => &event.stages_done,
        V::Continuing => &event.continuing_save,
        V::Current => &event.current_player,
        V::EventMode => &event.event_kind,
    }
}

/// Starts the event of a starter.
pub fn start_event(o: crate::owners::Owners<'static>, s: &Starter) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::menu::starters::start`. The players' own settings come back
    // afterwards (`menu::own_settings`).
    crate::menu::own_settings::around_championship(menus, || {
        starters::start(
            s.kind,
            &mut Game {
                menus,
                owners: o,
                cars: None,
            },
        )
    })
}
