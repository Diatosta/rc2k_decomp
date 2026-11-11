//! What the event flows share: the championship drivers
//! (`menu::run_championship`, `menu::run_a8_championship`), their starters, the single
//! rally's and the time trial's multi-player events. Each of them is a sequence of calls
//! and of tests of the game's variables. The flows run on `menu::flow_game::Game`
//! (the callees, the screens through `menu::screen_loop`).

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::records::stages_store::RecordRef;
use crate::session::{Dword, Flag};
use crate::system::owner::system;
use rc2k_core::menu::leaving::Switch;

/// A game variable.
pub fn get(v: &Dword) -> u32 {
    v.get()
}

pub fn set(v: &Dword, x: u32) {
    v.set(x);
}

pub fn inc(v: &Dword) {
    set(v, get(v).wrapping_add(1));
}

/// Reads for closures that a core function calls (they cannot return an error): the first
/// refusal is kept and the reads after it answer 0; the caller runs `check` right after the
/// core call, before it uses anything the core made.
#[derive(Default)]
pub struct Fault(std::cell::Cell<Option<BadInput>>);

impl Fault {
    /// The value, or the type's default after keeping the refusal.
    pub fn ok<T: Default>(&self, r: Result<T, BadInput>) -> T {
        r.unwrap_or_else(|e| {
            if self.0.get().is_none() {
                self.0.set(Some(e));
            }
            T::default()
        })
    }

    /// A `T` at the place `at`.
    pub fn read_at<T: crate::pointers::Plain + Default>(
        &self,
        at: crate::pointers::GamePtr,
        what: &str,
    ) -> T {
        self.ok(at.read::<T>(what))
    }

    pub fn check(&self) -> Result<(), BadInput> {
        self.0.take().map_or(Ok(()), Err)
    }
}

/// A refusal where the caller's signature cannot carry one (a trait of the core's, an
/// installer helper): the program ends naming it, as a refusal does at the top.
pub fn refuse_here<T>(r: Result<T, BadInput>, at: &str) -> T {
    match r {
        Ok(value) => value,
        Err(e) => crate::fatal::raise::end(at, e),
    }
}

// ---- the variables ----

pub const ANSWER: &Dword = &crate::menu::owner::MENUS.tables.pages.yes_no_answer; // g_yesNoAnswer
pub const CURRENT: &Dword = &crate::menu::owner::MENUS.screens.event.current_player; // the current player
pub const PLAYER_COUNT: &Dword = &crate::menu::owner::MENUS.screens.event.player_count;
pub const EVENT_MODE: &Dword = &crate::menu::owner::MENUS.screens.event.event_kind; // 9 .. 0x18, `menu::run_event_races`
pub const EVENT_TYPE: &Dword = &crate::menu::owner::MENUS.screens.event.event_type; // 3 time trial, 4, 5 the championships
pub const MODE: &Dword = &crate::menu::owner::MENUS.play.champ_view.event_mode; // g_eventMode: 2, 3, 0x13
pub const LEFT_BY_KEY: &Flag = &crate::menu::owner::MENUS.tables.captions.left_by_key; // the car choice was left
pub const EVENT_LEFT: &Flag = &crate::menu::owner::MENUS.screens.loop_flags.event_left;
pub const LEAVE_SCREEN: &Flag = &crate::menu::owner::MENUS.screens.loop_flags.leave_screen;
pub const LEFT_WITH_CARRY: &Flag = &crate::menu::owner::MENUS.screens.loop_flags.left_with_carry;
pub const ASK_LEAVE: &Flag = &crate::menu::owner::MENUS.screens.loop_flags.ask_leave;
pub const EVENT_OVER: &Flag = &crate::menu::owner::MENUS.screens.event.event_over; // 1 when an event is over

/// The variable of a leaving switch.
pub fn switch_cell(s: Switch) -> &'static Flag {
    match s {
        Switch::EventLeft => EVENT_LEFT,
        Switch::LeaveScreen => LEAVE_SCREEN,
        Switch::LeftWithCarry => LEFT_WITH_CARRY,
        Switch::AskLeave => ASK_LEAVE,
        Switch::EventOver => EVENT_OVER,
        Switch::LeftByKey => LEFT_BY_KEY,
        Switch::EscapePressed => ESCAPE_PRESSED,
        Switch::Entered => &crate::menu::owner::MENUS.screens.event.any_time_qualified,
        Switch::Quitting => &system().session.flags.quitting,
        Switch::RallyDone => &crate::menu::owner::MENUS.screens.state.menus.rally_done,
    }
}
pub const ESCAPE_PRESSED: &Flag = &crate::menu::owner::MENUS.screens.draw_work.escape_pressed;
pub const CONTINUING: &Dword = &crate::menu::owner::MENUS.screens.event.continuing_save; // 1 when a saved game goes on
pub const INDEX: &Dword = &crate::menu::owner::MENUS.screens.event.index; // index
pub const SECTION: &Dword = &crate::menu::owner::MENUS.screens.event.section; // section
pub const CHEATS: &Dword = &crate::menu::owner::MENUS.play.cheats.cheats; // g_cheats
pub const ROUND: &Dword = &crate::menu::owner::MENUS.play.results.round; // the rounds of a multi-player event

/// The car lists: `off_759CAE` (the championship's), `off_759D16` (the A8 championship's),
/// `off_759DCA` / `off_75A042` (by `dword_7554D8`), `off_75A412` (with cheat bit 1).
pub const CAR_LIST_KIND: &Dword = &crate::menu::owner::MENUS.screens.flow.car_list_kind; // 1 picks the first list

/// Player `n`'s record (`g_players`).
pub fn player(menus: &Menus, n: u32) -> crate::menu::standings::PlayerCell<'_> {
    &menus.play.players.players[n as usize]
}

pub fn yes() -> bool {
    get(ANSWER) == 1
}

/// The current player's record, refused past the four.
pub fn current_player(menus: &Menus) -> Result<crate::menu::standings::PlayerCell<'_>, BadInput> {
    let n = get(CURRENT);
    if n >= 4 {
        return Err(BadInput("a current player past the four"));
    }
    Ok(player(menus, n))
}

/// The stage chosen: `off_735512[dword_7558C8][dword_7558D0]` + `dword_7558CC` x 0xA4.
pub const STAGE_KIND: &Dword = &crate::menu::owner::MENUS.screens.event.stage_group;
pub const STAGE_SUB: &Dword = &crate::menu::owner::MENUS.screens.event.stage_sub;
pub const STAGE_NUMBER: &Dword = &crate::menu::owner::MENUS.screens.event.stage_number;
const KINDS: u32 = 4;
/// The stages' best times: `g_stageBestTimesAlt` (kind 1) or `dword_7556E8`, ten for each
/// stage (the record's +0x3C).
pub type StageTable = crate::memory::owned::Owned<rc2k_core::saves::entries::StageTimes>;
pub const TIMES_ONE: &StageTable = &crate::saves::best_times::STAGE_TIMES_ALT;
pub const TIMES_OTHER: &StageTable = &crate::saves::best_times::TIME_TRIAL_TIMES;

/// The chosen stage's record; refused for a kind past the table's four or a null pointer
/// on the way (the original reads through it).
pub fn stage_record() -> Result<RecordRef, BadInput> {
    let kind = get(STAGE_KIND);
    if kind >= KINDS {
        return Err(BadInput("a stage kind past the event stage tables' four"));
    }
    let stages = system()
        .records
        .stage_tables
        .group_event_table(kind, get(STAGE_SUB))?;
    crate::records::stages_store::row(stages, get(STAGE_NUMBER))
}

/// The chosen stage's best times: its table and its place there; refused as
/// `stage_record`, or with no table or a stage past it.
pub fn stage_times() -> Result<(&'static StageTable, usize), BadInput> {
    let record = stage_record()?;
    let table = if get(STAGE_KIND) == 1 {
        TIMES_ONE
    } else {
        TIMES_OTHER
    };
    let stage = record.record()?.event_slot() as usize;
    if !table.with(|t| stage < t.stages.len())? {
        return Err(BadInput("a stage past its best times table"));
    }
    Ok((table, stage))
}

/// `f` on the chosen stage's ten best times; refused as `stage_times`.
pub fn with_stage_times<R>(
    f: impl FnOnce(&mut rc2k_core::saves::entries::Times) -> R,
) -> Result<R, BadInput> {
    let (table, stage) = stage_times()?;
    table.with(|t| f(&mut t.stages[stage]))
}
