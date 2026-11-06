//! What the event flows share (IDA, 2026-10-19): the championship drivers
//! (`menu::run_championship`, `menu::run_a8_championship`), their starters, the single
//! rally's and the time trial's multi-player events. Each of them is a sequence of calls
//! and of tests of the game's variables. The flows run on `menu::flow_game::Game`
//! (the callees' Rust versions, the screens through `menu::screen_loop`).

use crate::errors::BadInput;
use crate::records::stages_store::RecordRef;
use crate::session::Dword;

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
pub const CURRENT: &Dword = &crate::menu::owner::MENUS.screens.event.current_player; // dword_75598C: the current player
pub const PLAYER_COUNT: &Dword = &crate::menu::owner::MENUS.screens.event.player_count; // dword_755984
pub const EVENT_MODE: &Dword = &crate::menu::owner::MENUS.screens.event.event_kind; // dword_755988 (9 .. 0x18, `menu::run_event_races`)
pub const EVENT_TYPE: &Dword = &crate::menu::owner::MENUS.screens.event.event_type; // dword_755954: 3 time trial, 4, 5 the championships
pub const MODE: &Dword = &crate::menu::owner::MENUS.play.champ_view.event_mode; // g_eventMode: 2, 3, 0x13
pub const CAR_LEFT: &Dword = &crate::menu::owner::MENUS.tables.captions.screen_left; // dword_758806: the car choice was left
pub const LEAVE: &Dword = &crate::menu::owner::MENUS.screens.loop_flags.went_back; // dword_75555D
pub const LEAVE_561: &Dword = &crate::menu::owner::MENUS.screens.loop_flags.back; // dword_755561
pub const ENDED_565: &Dword = &crate::menu::owner::MENUS.screens.loop_flags.ended_b; // dword_755565
pub const ENDED_569: &Dword = &crate::menu::owner::MENUS.screens.loop_flags.ended_a; // dword_755569
pub const DONE: &Dword = &crate::menu::owner::MENUS.screens.event.event_done; // dword_755980: 1 when an event is over
pub const LEFT_A3C: &Dword = &crate::menu::owner::MENUS.screens.draw_work.escaped; // dword_755A3C
pub const CONTINUING: &Dword = &crate::menu::owner::MENUS.screens.event.continuing_save; // dword_755970: 1 when a saved game goes on
pub const INDEX: &Dword = &crate::menu::owner::MENUS.screens.event.index; // index
pub const SECTION: &Dword = &crate::menu::owner::MENUS.screens.event.section; // section
pub const CHEATS: &Dword = &crate::menu::owner::MENUS.play.cheats.cheats; // g_cheats
pub const ROUND: &Dword = &crate::menu::owner::MENUS.play.results.round; // dword_758714: the rounds of a multi-player event

/// The car lists: `off_759CAE` (the championship's), `off_759D16` (the A8 championship's),
/// `off_759DCA` / `off_75A042` (by `dword_7554D8`), `off_75A412` (with cheat bit 1).
pub const CAR_LIST_KIND: &Dword = &crate::menu::owner::MENUS.screens.flow.car_list_kind; // dword_7554D8: 1 picks the first list

/// Player `n`'s record (`g_players`).
pub fn player(n: u32) -> crate::menu::standings::PlayerCell {
    &crate::menu::owner::menus().play.players.players[n as usize]
}

pub fn yes() -> bool {
    get(ANSWER) == 1
}

/// The current player's record, refused past the four.
pub fn current_player() -> Result<crate::menu::standings::PlayerCell, BadInput> {
    let n = get(CURRENT);
    if n >= 4 {
        return Err(BadInput("a current player past the four"));
    }
    Ok(player(n))
}

/// The stage chosen: `off_735512[dword_7558C8][dword_7558D0]` + `dword_7558CC` x 0xA4.
pub const STAGE_KIND: &Dword = &crate::menu::owner::MENUS.screens.event.stage_group; // dword_7558C8
pub const STAGE_SUB: &Dword = &crate::menu::owner::MENUS.screens.event.stage_sub; // dword_7558D0
pub const STAGE_NUMBER: &Dword = &crate::menu::owner::MENUS.screens.event.stage_number; // dword_7558CC
const KINDS: u32 = 4;
/// The stages' best times: `g_stageBestTimesAlt` (kind 1) or `dword_7556E8`, ten for each
/// stage (the record's +0x3C).
pub type StageTable = crate::memory::owned::Owned<rc2k_core::saves::entries::StageTimes>;
pub const TIMES_ONE: &StageTable = &crate::saves::best_times::STAGE_TIMES_ALT; // dword_7556E4
pub const TIMES_OTHER: &StageTable = &crate::saves::best_times::TIME_TRIAL_TIMES; // dword_7556E8

/// The chosen stage's record; refused for a kind past the table's four or a null pointer
/// on the way (the original reads through it).
pub fn stage_record() -> Result<RecordRef, BadInput> {
    let kind = get(STAGE_KIND);
    if kind >= KINDS {
        return Err(BadInput("a stage kind past off_735512's four"));
    }
    let stages =
        crate::records::stages_store::STAGE_TABLES.group_event_table(kind, get(STAGE_SUB))?;
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
