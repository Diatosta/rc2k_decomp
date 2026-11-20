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
use crate::memory::session::PlainDword;
use crate::menu::event_flow::*;
use crate::menu::event_store::EventWord;
use crate::menu::lists_store::CarList;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};
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
    /// A launch's answers to the car choices.
    answer: Option<crate::menu::launch::CarAnswer<'a>>,
}

impl starters::Event for Game<'_> {
    type Error = Refused;

    fn get(&mut self, v: starters::Var) -> u32 {
        match var(self.menus, v) {
            Slot::Event(word) => word.get(&self.menus.screens.event),
            Slot::Cell(cell) => get(cell),
        }
    }

    fn set(&mut self, v: starters::Var, value: u32) {
        match var(self.menus, v) {
            Slot::Event(word) => word.set(&self.menus.screens.event, value),
            Slot::Cell(cell) => set(cell, value),
        }
    }

    fn flag(&mut self, s: Switch) -> bool {
        switch_is_set(self.owners.menus, self.owners.system, s)
    }

    fn set_flag(&mut self, s: Switch, on: bool) {
        set_switch(self.owners.menus, self.owners.system, s, on);
    }

    fn call(&mut self, c: starters::Call) -> Result<(), Refused> {
        use crate::menu as m;
        use starters::Call as C;
        match c {
            C::RepairCar => crate::service::repair_parts::repair_car_in(self.menus)?,
            C::BeginEvent => m::begin_event::begin_event(
                &self.menus.cars,
                &self.menus.play,
                &self.menus.screens,
            )?,
            C::ResetDrivers => m::reset_drivers::reset_drivers(self.menus)?,
            C::ResetDriverCondition => {
                m::reset_driver_condition::reset_driver_condition(self.menus)?;
            }
            C::SelectCar(list) => {
                self.cars = Some(list);
                let answered = match self.answer.as_mut() {
                    Some(answer) => answer(list)?,
                    None => None,
                };
                match answered {
                    Some(chosen_in) => {
                        self.cars = Some(chosen_in);
                        starters::Event::set_flag(self, Switch::LeftByKey, false);
                    }
                    None => {
                        run_screen(
                            ScreenId::SelectCar,
                            &Entry::with_cars(self.owners, self.cars),
                        )?;
                    }
                }
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

/// Where a starter's variable is.
enum Slot<'a> {
    Event(EventWord),
    Cell(&'a PlainDword),
}

fn var(menus: &crate::menu::owner::Menus, v: starters::Var) -> Slot<'_> {
    use starters::Var as V;
    match v {
        V::EventType => Slot::Event(EventWord::EventType),
        V::RepairSet => Slot::Cell(&menus.cars.table.last_car), // 0xF
        V::PlayerCount => Slot::Event(EventWord::PlayerCount),
        V::Index => Slot::Event(EventWord::StageInSection),
        V::Section => Slot::Event(EventWord::Section),
        V::StagesDone => Slot::Event(EventWord::StagesDone),
        V::Continuing => Slot::Event(EventWord::ContinuingSave),
        V::Current => Slot::Event(EventWord::CurrentPlayer),
        V::EventMode => Slot::Event(EventWord::EventKind),
    }
}

/// Starts the event of a starter.
pub(super) fn start_event(o: crate::owners::Owners<'static>, s: &Starter) -> Result<(), Refused> {
    start_event_with(o, s, None)
}

/// `start_event` with a launch's answers to the car choices.
pub(super) fn start_event_with<'a>(
    o: crate::owners::Owners<'static>,
    s: &Starter,
    answer: Option<crate::menu::launch::CarAnswer<'a>>,
) -> Result<(), Refused> {
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
                answer,
            },
        )
    })
}
