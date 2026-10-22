//! The championships' starters (IDA, 2026-10-19): the actions of the start menus' items
//! that set a championship up for one to four players and run it. Ten of them, five for
//! each championship, alike but for their numbers; each is a hook of its own
//! (`menu::start_championship_*`, `menu::start_a8_*`), this is what they share.
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
use crate::framework::{Regs, SelfTest};
use crate::menu::championship_welcome::addr;
use crate::menu::event_flow::*;
use rc2k_core::menu::starters;

const REPAIR_SET: usize = 0x75649C; // dword_75649C: 0xF

pub struct Starter {
    pub address: usize,
    pub len: usize,
    pub entry: &'static [u8],
    pub calls: &'static [usize],
    /// The championship and the seats (see `rc2k_core::menu::starters`).
    pub kind: starters::Starter,
}

/// The game's variables and callees for `rc2k_core::menu::starters::start`, with the
/// registers the original has at each call.
struct Game<'a> {
    g: Regs,
    call: Call<'a>,
}

impl starters::Event for Game<'_> {
    type Error = Refused;

    fn get(&mut self, v: starters::Var) -> u32 {
        get(var(v))
    }

    fn set(&mut self, v: starters::Var, value: u32) {
        set(var(v), value);
    }

    fn call(&mut self, c: starters::Call) -> Result<(), Refused> {
        use starters::Call as C;
        let callee = match c {
            C::RepairCar { current } => {
                if current {
                    self.g.edi = player(get(CURRENT));
                }
                REPAIR_CAR
            }
            C::BeginEvent => BEGIN_EVENT,
            C::ResetDrivers => RESET_DRIVERS,
            C::ResetDriverCondition => RESET_DRIVER_CONDITION,
            C::SelectCar(list) => {
                self.g.esi = addr(match list {
                    starters::CarList::Championship => CARS_CHAMPIONSHIP,
                    starters::CarList::A8 => CARS_A8,
                });
                SELECT_CAR_LOOP
            }
            C::TakeShownCar => TAKE_SHOWN_CAR,
            C::ResetCars750724 => RESET_CARS_750724,
            C::RunA8Championship => RUN_A8_CHAMPIONSHIP,
            C::ShowInGameOptions => SHOW_IN_GAME_OPTIONS,
            C::RunChampionship => RUN_CHAMPIONSHIP,
        };
        (self.call)(callee, &mut self.g)
    }
}

fn var(v: starters::Var) -> usize {
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
pub unsafe fn start_event(s: &Starter, entry: &Regs, call: Call) -> Result<Regs, Refused> {
    players_there(s.kind.players())?;
    // Converted: `rc2k_core::menu::starters::start`.
    starters::start(s.kind, &mut Game { g: *entry, call })?;
    Ok(*entry)
}

/// A starter's self-test: every car choice left in turn and none, both ways of a saved
/// game for the A8's first.
pub fn test_starter(t: SelfTest, s: &'static Starter) -> (usize, usize) {
    if (0..s.kind.players()).any(|k| player(k) == 0) {
        return t.skip("a player missing from g_players");
    }
    let flow = Flow {
        address: s.address,
        len: s.len,
        entry: s.entry,
        calls: s.calls,
        fatal_jumps: &[],
        writes: &[
            EVENT_TYPE,
            CAR_LEFT,
            REPAIR_SET,
            INDEX,
            SECTION,
            crate::menu::run_championship::STAGES_DONE,
            PLAYER_COUNT,
            CURRENT,
            EVENT_MODE,
            LEAVE,
            ENDED_569,
            ENDED_565,
            DONE,
            CONTINUING,
        ],
    };
    let continuing: &[u32] = if s.kind.first() && s.kind.championship == starters::Championship::A8
    {
        &[0, 1]
    } else {
        &[0]
    };
    let prepares: Vec<Box<dyn Fn()>> = continuing
        .iter()
        .map(|&c| Box::new(move || set(CONTINUING, c)) as Box<dyn Fn()>)
        .collect();
    let mut cases = Vec::new();
    for (prepare, &c) in prepares.iter().zip(continuing) {
        for left in 0..=s.kind.players() as usize {
            cases.push(Case {
                what: format!("continuing {c}, the car choice left at {left}"),
                prepare: prepare.as_ref(),
                script: vec![(
                    SELECT_CAR_LOOP,
                    car_choices(s.kind.players() as usize, left),
                )],
                before: Regs::SENTINEL,
            });
        }
    }
    test_flow(t, &flow, &[], &cases, &|r, c| unsafe {
        start_event(s, r, c)
    })
}

/// A starter's hook file: the hook, its dispatch and its test.
macro_rules! starter_hook {
    ($name:literal, $starter:ident, $($patch:tt)*) => {
        /// The Rust version where the typed tables run it (`menu::table_calls`; hook removed
        /// 2026-10-22).
        pub fn entry(r: &mut $crate::framework::Regs) -> Result<(), $crate::errors::Refused> {
            let left = unsafe {
                $crate::menu::start_event::start_event(
                    &$starter,
                    r,
                    &mut $crate::menu::event_flow::game_call,
                )
            }?;
            *r = $crate::framework::Regs { esp: r.esp, ..left };
            Ok(())
        }
    };
}
pub(crate) use starter_hook;
