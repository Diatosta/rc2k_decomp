//! The championships' starters: the start menus' items that set a championship up for
//! one to four players and run it (from `menu::start_championship_one`
//! 0x4A25F4, `_split`, `_two` 0x4A27B5, `_three`, `_four`, and `menu::start_a8_one`
//! 0x4A9C9B, `_split`, `_two`, `_three`, `_four`). Their steps in the original's order, on
//! the game's variables and callees through `Event`.

use super::leaving::Switch;

/// The two championships.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Championship {
    /// `g_eventType` 4, `menu::run_championship`.
    First,
    /// `g_eventType` 5, `menu::run_a8_championship`.
    A8,
}

/// The seats: how many players, and the split screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Seats {
    One,
    Split,
    Two,
    Three,
    Four,
}

/// A starter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Starter {
    pub championship: Championship,
    pub seats: Seats,
}

impl Starter {
    /// `dword_755954`.
    pub fn event_type(self) -> u32 {
        match self.championship {
            Championship::First => 4,
            Championship::A8 => 5,
        }
    }

    pub fn players(self) -> u32 {
        match self.seats {
            Seats::One => 1,
            Seats::Split | Seats::Two => 2,
            Seats::Three => 3,
            Seats::Four => 4,
        }
    }

    /// `dword_755988`, the event mode (`menu::run_event_races`).
    pub fn mode(self) -> u32 {
        match self.seats {
            Seats::One => 9,
            Seats::Split => 0xA,
            Seats::Two => 0xB,
            Seats::Three => 0xC,
            Seats::Four => 0xD,
        }
    }

    /// The one-player starters differ (see `start`).
    pub fn first(self) -> bool {
        self.seats == Seats::One
    }

    fn a8(self) -> bool {
        self.championship == Championship::A8
    }
}

/// The game's variables a starter writes or reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Var {
    /// `dword_755954`.
    EventType,
    /// `dword_75649C`.
    RepairSet,
    /// `dword_755984`.
    PlayerCount,
    /// `index`, `section`.
    Index,
    Section,
    /// `dword_755950`.
    StagesDone,
    /// `dword_755970`: 1 when a saved game goes on.
    Continuing,
    /// The current player.
    Current,
    /// `dword_755988`.
    EventMode,
}

pub use super::car_lists::CarList;

/// The callees.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    /// `service::repair_car`: the current player's car.
    RepairCar,
    BeginEvent,
    ResetDrivers,
    ResetDriverCondition,
    /// `menu::select_car_loop` with the list.
    SelectCar(CarList),
    TakeShownCar,
    HideAllInGameOptions,
    RunA8Championship,
    ShowInGameOptions,
    RunChampionship,
}

/// The game a starter runs on.
pub trait Event {
    type Error;
    fn get(&mut self, v: Var) -> u32;
    fn set(&mut self, v: Var, value: u32);
    fn flag(&mut self, s: Switch) -> bool;
    fn set_flag(&mut self, s: Switch, on: bool);
    fn call(&mut self, c: Call) -> Result<(), Self::Error>;
}

/// The repairs' set.
pub const ALL_REPAIRS: u32 = 0xF;

/// A starter's steps: the set-up (the one-player ones in their own order; the A8's
/// first begins the event unless a saved game goes on), each player's repairs and car
/// choice (leaving it ends everything), the event mode, the championship, its ending.
pub fn start<E: Event>(s: Starter, e: &mut E) -> Result<(), E::Error> {
    let list = if s.a8() {
        CarList::A8
    } else {
        CarList::Championship
    };
    let zero_round = |e: &mut E| {
        e.set(Var::Index, 0);
        e.set(Var::Section, 0);
        e.set(Var::StagesDone, 0);
    };
    match (s.first(), s.a8()) {
        (true, false) => {
            e.call(Call::RepairCar)?;
            e.set_flag(Switch::LeftByKey, false);
            e.set(Var::EventType, s.event_type());
            e.set(Var::RepairSet, ALL_REPAIRS);
            e.set(Var::PlayerCount, 1);
            zero_round(e);
        }
        (true, true) => {
            e.set(Var::EventType, s.event_type());
            e.set_flag(Switch::LeftByKey, false);
            e.set(Var::PlayerCount, 1);
            if e.get(Var::Continuing) != 1 {
                e.call(Call::BeginEvent)?;
                e.call(Call::ResetDrivers)?;
                e.call(Call::ResetDriverCondition)?;
            }
            e.set(Var::Continuing, 0);
            zero_round(e);
        }
        (false, _) => {
            e.set(Var::EventType, s.event_type());
            e.set_flag(Switch::LeftByKey, false);
            e.set(Var::RepairSet, ALL_REPAIRS);
            e.call(Call::ResetDriverCondition)?;
            zero_round(e);
            e.set(Var::PlayerCount, s.players());
        }
    }
    for k in 0..s.players() {
        e.set(Var::Current, k);
        e.call(Call::RepairCar)?;
        e.call(Call::SelectCar(list))?;
        if e.flag(Switch::LeftByKey) {
            return Ok(());
        }
        e.call(Call::TakeShownCar)?;
    }
    e.set(Var::EventMode, s.mode());
    if s.a8() {
        if s.first() {
            e.call(Call::HideAllInGameOptions)?;
        }
        e.call(Call::RunA8Championship)?;
        if s.first() {
            e.call(Call::ShowInGameOptions)?;
        }
    } else {
        e.call(Call::RunChampionship)?;
        e.set_flag(Switch::EventLeft, false);
        e.set_flag(Switch::AskLeave, false);
        e.set_flag(Switch::LeftWithCarry, false);
    }
    e.set_flag(Switch::EventOver, true);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Log {
        steps: Vec<String>,
        continuing: u32,
        leave_at: Option<u32>,
        chosen: u32,
    }

    impl Event for Log {
        type Error = ();
        fn get(&mut self, v: Var) -> u32 {
            match v {
                Var::Continuing => self.continuing,
                _ => unreachable!(),
            }
        }
        fn set(&mut self, v: Var, value: u32) {
            self.steps.push(format!("{v:?}={value}"));
        }
        fn flag(&mut self, s: Switch) -> bool {
            match s {
                Switch::LeftByKey => self.leave_at == Some(self.chosen - 1),
                _ => unreachable!(),
            }
        }
        fn set_flag(&mut self, s: Switch, on: bool) {
            self.steps.push(format!("{s:?}={}", u32::from(on)));
        }
        fn call(&mut self, c: Call) -> Result<(), ()> {
            if let Call::SelectCar(_) = c {
                self.chosen += 1;
            }
            self.steps.push(format!("{c:?}"));
            Ok(())
        }
    }

    #[test]
    fn starters() {
        let two = Starter {
            championship: Championship::First,
            seats: Seats::Two,
        };
        assert_eq!((two.event_type(), two.players(), two.mode()), (4, 2, 0xB));
        let mut log = Log::default();
        start(two, &mut log).unwrap();
        assert_eq!(log.steps.first().unwrap(), "EventType=4");
        assert_eq!(log.steps.iter().filter(|s| *s == "TakeShownCar").count(), 2);
        assert_eq!(log.steps.last().unwrap(), "EventOver=1");
        assert!(log.steps.contains(&"RunChampionship".to_string()));

        let a8 = Starter {
            championship: Championship::A8,
            seats: Seats::One,
        };
        let mut log = Log {
            continuing: 1,
            ..Log::default()
        };
        start(a8, &mut log).unwrap();
        assert!(!log.steps.contains(&"BeginEvent".to_string()));
        assert!(log.steps.contains(&"ShowInGameOptions".to_string()));
        assert!(log.steps.contains(&"SelectCar(A8)".to_string()));

        let mut log = Log {
            leave_at: Some(1),
            ..Log::default()
        };
        let four = Starter {
            championship: Championship::First,
            seats: Seats::Four,
        };
        start(four, &mut log).unwrap();
        assert_eq!(log.steps.iter().filter(|s| *s == "TakeShownCar").count(), 1);
        assert!(!log.steps.contains(&"EventOver=1".to_string()));
    }
}
