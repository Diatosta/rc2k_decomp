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

const REPAIR_SET: usize = 0x75649C; // dword_75649C: 0xF

pub struct Starter {
    pub address: usize,
    pub len: usize,
    pub entry: &'static [u8],
    pub calls: &'static [usize],
    /// 4 the championship, 5 the A8 championship.
    pub event_type: u32,
    pub players: u32,
    /// `dword_755988`.
    pub mode: u32,
    /// The one-player starter: 0x4A25F4 (the repairs first, the variables in another
    /// order) or 0x4A9C9B (no `dword_75649C`; unless a saved game goes on `begin_event`,
    /// `reset_drivers`, `reset_driver_condition`; `dword_755970` 0; `reset_cars_750724`
    /// before the A8 championship and `show_in_game_options` after it).
    pub first: bool,
}

impl Starter {
    fn a8(&self) -> bool {
        self.event_type == 5
    }
}

/// The Rust version of a starter.
pub unsafe fn start_event(s: &Starter, entry: &Regs, call: Call) -> Result<Regs, Refused> {
    players_there(s.players)?;
    let mut g = *entry;
    let list = if s.a8() { CARS_A8 } else { CARS_CHAMPIONSHIP };
    match (s.first, s.a8()) {
        (true, false) => {
            call(REPAIR_CAR, &mut g)?;
            set(CAR_LEFT, 0);
            set(EVENT_TYPE, s.event_type);
            set(REPAIR_SET, 0xF);
            set(PLAYER_COUNT, 1);
            set(INDEX, 0);
            set(SECTION, 0);
            set(crate::menu::run_championship::STAGES_DONE, 0);
        }
        (true, true) => {
            set(EVENT_TYPE, s.event_type);
            set(CAR_LEFT, 0);
            set(PLAYER_COUNT, 1);
            if get(CONTINUING) != 1 {
                call(BEGIN_EVENT, &mut g)?;
                call(RESET_DRIVERS, &mut g)?;
                call(RESET_DRIVER_CONDITION, &mut g)?;
            }
            set(CONTINUING, 0);
            set(INDEX, 0);
            set(SECTION, 0);
            set(crate::menu::run_championship::STAGES_DONE, 0);
        }
        (false, _) => {
            set(EVENT_TYPE, s.event_type);
            set(CAR_LEFT, 0);
            set(REPAIR_SET, 0xF);
            call(RESET_DRIVER_CONDITION, &mut g)?;
            set(INDEX, 0);
            set(SECTION, 0);
            set(crate::menu::run_championship::STAGES_DONE, 0);
            set(PLAYER_COUNT, s.players);
        }
    }
    for k in 0..s.players {
        set(CURRENT, k);
        g.edi = player(get(CURRENT));
        call(REPAIR_CAR, &mut g)?;
        g.esi = addr(list);
        call(SELECT_CAR_LOOP, &mut g)?;
        if get(CAR_LEFT) == 1 {
            return Ok(*entry);
        }
        call(TAKE_SHOWN_CAR, &mut g)?;
    }
    set(EVENT_MODE, s.mode);
    if s.a8() {
        if s.first {
            call(RESET_CARS_750724, &mut g)?;
        }
        call(RUN_A8_CHAMPIONSHIP, &mut g)?;
        if s.first {
            call(SHOW_IN_GAME_OPTIONS, &mut g)?;
        }
    } else {
        call(RUN_CHAMPIONSHIP, &mut g)?;
        set(LEAVE, 0);
        set(ENDED_569, 0);
        set(ENDED_565, 0);
    }
    set(DONE, 1);
    Ok(*entry)
}

/// A starter's self-test: every car choice left in turn and none, both ways of a saved
/// game for the A8's first.
pub fn test_starter(t: SelfTest, s: &'static Starter) -> (usize, usize) {
    if (0..s.players).any(|k| player(k) == 0) {
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
    let continuing: &[u32] = if s.first && s.a8() { &[0, 1] } else { &[0] };
    let prepares: Vec<Box<dyn Fn()>> = continuing
        .iter()
        .map(|&c| Box::new(move || set(CONTINUING, c)) as Box<dyn Fn()>)
        .collect();
    let mut cases = Vec::new();
    for (prepare, &c) in prepares.iter().zip(continuing) {
        for left in 0..=s.players as usize {
            cases.push(Case {
                what: format!("continuing {c}, the car choice left at {left}"),
                prepare: prepare.as_ref(),
                script: vec![(SELECT_CAR_LOOP, car_choices(s.players as usize, left))],
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
        static ORIGINAL: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

        pub static HOOK: $crate::framework::RawHook = $crate::framework::RawHook::new(
            $name,
            $crate::framework::rva($starter.address),
            thunk,
            &ORIGINAL,
        )
        $($patch)*
        .self_test(self_test);

        $crate::framework::hook_asm!(
            thunk,
            call_original,
            dispatch = dispatch,
            original = ORIGINAL,
            ungated
        );

        extern "C" fn dispatch(r: &mut $crate::framework::Regs) {
            HOOK.count_call();
            if $crate::framework::shadow() {
                unsafe { call_original(r) };
                return;
            }
            match unsafe {
                $crate::menu::start_event::start_event(
                    &$starter,
                    r,
                    &mut $crate::menu::event_flow::game_call,
                )
            } {
                Ok(left) => *r = $crate::framework::Regs { esp: r.esp, ..left },
                Err(e) => unsafe { $crate::fatal::raise::refused(&HOOK, e, r, call_original) },
            }
        }

        fn self_test() -> (usize, usize) {
            let Some(t) = $crate::framework::SelfTest::begin(&HOOK) else {
                return (0, 0);
            };
            $crate::menu::start_event::test_starter(t, &$starter)
        }
    };
}
pub(crate) use starter_hook;
