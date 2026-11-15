//! What the event flows share: the championship drivers
//! (`menu::run_championship`, `menu::run_a8_championship`), their starters, the single
//! rally's and the time trial's multi-player events. Each of them is a sequence of calls
//! and of tests of the game's variables. The flows run on `menu::flow_game::Game`
//! (the callees, the screens through `menu::screen_loop`).

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::records::stages_store::RecordRef;
use crate::session::FlagLike;
use crate::session::PlainDword;
use crate::system::owner::System;
use rc2k_core::menu::leaving::Switch;

/// A game variable.
pub fn get(v: &PlainDword) -> u32 {
    v.get()
}

pub fn set(v: &PlainDword, x: u32) {
    v.set(x);
}

pub fn inc(v: &PlainDword) {
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

/// The variable of a leaving switch.
pub fn switch_cell<'a>(menus: &'a Menus, system: &'a System, s: Switch) -> &'a dyn FlagLike {
    let screens = &menus.screens;
    match s {
        Switch::EventLeft => &screens.loop_flags.event_left,
        Switch::LeaveScreen => &screens.loop_flags.leave_screen,
        Switch::LeftWithCarry => &screens.loop_flags.left_with_carry,
        Switch::AskLeave => &screens.loop_flags.ask_leave,
        Switch::EventOver => &screens.event.event_over,
        Switch::LeftByKey => &menus.tables.captions.left_by_key,
        Switch::EscapePressed => &screens.draw_work.escape_pressed,
        Switch::Entered => &screens.event.any_time_qualified,
        Switch::Quitting => &system.session.flags.quitting,
        Switch::RallyDone => &screens.state.menus.rally_done,
    }
}

/// Player `n`'s record.
pub fn player(menus: &Menus, n: u32) -> crate::menu::standings::PlayerCell<'_> {
    &menus.play.slots.records[n as usize]
}

pub fn yes(menus: &Menus) -> bool {
    menus.tables.pages.yes_no_answer.get() == 1
}

/// The current player's record, refused past the four.
pub fn current_player(menus: &Menus) -> Result<crate::menu::standings::PlayerCell<'_>, BadInput> {
    let n = menus.screens.event.current_player.get();
    if n >= 4 {
        return Err(BadInput("a current player past the four"));
    }
    Ok(player(menus, n))
}

/// The stage chosen: the stage table of `Event.stage_group` and `Event.stage_sub`, then
/// `Event.stage_number` x 0xA4 into it.
const KINDS: u32 = 4;
/// The stages' best times: the second stage best times (kind 1) or
/// `ChampView.time_trial_best_times`, ten for each stage (the record's +0x3C).
pub type StageTable = crate::memory::owned::Owned<rc2k_core::saves::entries::StageTimes>;
pub const TIMES_ONE: &StageTable = &crate::saves::best_times::STAGE_TIMES_ALT;
pub const TIMES_OTHER: &StageTable = &crate::saves::best_times::TIME_TRIAL_TIMES;

/// The chosen stage's record; refused for a kind past the table's four or a null pointer
/// on the way.
pub fn stage_record(menus: &Menus, system: &System) -> Result<RecordRef, BadInput> {
    let event = &menus.screens.event;
    let kind = event.stage_group.get();
    if kind >= KINDS {
        return Err(BadInput("a stage kind past the event stage tables' four"));
    }
    let stages = system
        .records()
        .stage_tables
        .group_event_table(kind, event.stage_sub.get())?;
    crate::records::stages_store::row(
        &system.records().stage_tables,
        stages,
        event.stage_number.get(),
    )
}

/// The chosen stage's best times: its table and its place there; refused as
/// `stage_record`, or with no table or a stage past it.
pub fn stage_times(
    menus: &Menus,
    system: &System,
) -> Result<(&'static StageTable, usize), BadInput> {
    let record = stage_record(menus, system)?;
    let table = if menus.screens.event.stage_group.get() == 1 {
        TIMES_ONE
    } else {
        TIMES_OTHER
    };
    let stage = record.record(&system.records().stage_tables)?.event_slot() as usize;
    if !table.with(|t| stage < t.stages.len())? {
        return Err(BadInput("a stage past its best times table"));
    }
    Ok((table, stage))
}

/// `f` on the chosen stage's ten best times; refused as `stage_times`.
pub fn with_stage_times<R>(
    menus: &Menus,
    system: &System,
    f: impl FnOnce(&mut rc2k_core::saves::entries::Times) -> R,
) -> Result<R, BadInput> {
    let (table, stage) = stage_times(menus, system)?;
    table.with(|t| f(&mut t.stages[stage]))
}
