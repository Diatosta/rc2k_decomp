//! What the event flows share: the championship drivers
//! (`menu::run_championship`, `menu::run_a8_championship`), their starters, the single
//! rally's and the time trial's multi-player events. Each of them is a sequence of calls
//! and of tests of the game's variables. The flows run on `menu::flow_game::Game`
//! (the callees, the screens through `menu::screen_loop`).

use crate::errors::BadInput;
use crate::memory::session::FlagLike;
use crate::memory::session::PlainDword;
use crate::menu::owner::Menus;
use crate::menu::pages_store::MenuPages;
use crate::records::stages_store::RecordRef;
use crate::saves::best_times::BestTables;
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

/// A refusal where the caller's signature cannot carry one (a trait of the core's, an
/// installer helper): the program ends naming it, as a refusal does at the top.
pub fn refuse_here<T>(r: Result<T, BadInput>, at: &str) -> T {
    match r {
        Ok(value) => value,
        Err(e) => crate::fatal::raise::end(at, e),
    }
}

/// The variable of a leaving switch.
pub(super) fn switch_cell<'a>(menus: &'a Menus, system: &'a System, s: Switch) -> &'a dyn FlagLike {
    let screens = &menus.screens;
    match s {
        Switch::EventLeft => &screens.loop_flags.event_left,
        Switch::LeaveScreen => &screens.loop_flags.leave_screen,
        Switch::LeftWithCarry => &screens.loop_flags.left_with_carry,
        Switch::AskLeave => &screens.loop_flags.ask_leave,
        Switch::EventOver => &screens.event.event_over,
        Switch::LeftByKey => &menus.tables.captions.left_by_key,
        Switch::EscapePressed => &screens.draw_work.escape_pressed,
        Switch::Entered => &screens.event.trial.any_qualified,
        Switch::Quitting => &system.session.flags.quitting,
        Switch::RallyDone => &screens.state.menus.rally_done,
    }
}

/// Player `n`'s record.
pub fn player(menus: &Menus, n: u32) -> crate::menu::standings::PlayerCell<'_> {
    &menus.play.slots.records[n as usize]
}

pub fn yes(pages: &MenuPages) -> bool {
    pages.yes_no_answer.get() == 1
}

/// The current player's record, refused past the four.
pub fn current_player(menus: &Menus) -> Result<crate::menu::standings::PlayerCell<'_>, BadInput> {
    let n = menus.screens.event.current_player.get();
    if n >= 4 {
        return Err(BadInput("a current player past the four"));
    }
    Ok(player(menus, n))
}

/// The stage chosen: the stage table of `EventState.stage.group` and `EventState.stage.sub`, then
/// `EventState.stage.number` x 0xA4 into it.
const KINDS: u32 = 4;
/// The stages' best times: the second stage best times (kind 1) or
/// `ChampView.time_trial_best_times`, ten for each stage (the record's +0x3C).
pub type StageTable = crate::memory::owned::Owned<rc2k_core::saves::entries::StageTimes>;

/// The chosen stage's record; refused for a kind past the table's four or a null pointer
/// on the way.
pub fn stage_record(menus: &Menus, system: &System) -> Result<RecordRef, BadInput> {
    let event = &menus.screens.event;
    let kind = event.stage.group.get();
    if kind >= KINDS {
        return Err(BadInput("a stage kind past the event stage tables' four"));
    }
    let stages = system
        .records()
        .stage_tables
        .group_event_table(kind, event.stage.sub.get())?;
    crate::records::stages_store::row(
        &system.records().stage_tables,
        stages,
        event.stage.number.get(),
    )
}

/// The chosen stage's best times: its table and its place there; refused as
/// `stage_record`, or with no table or a stage past it.
pub fn stage_times<'t>(
    menus: &Menus,
    system: &System,
    tables: &'t BestTables,
) -> Result<(&'t StageTable, usize), BadInput> {
    let record = stage_record(menus, system)?;
    let table = if menus.screens.event.stage.group.get() == 1 {
        &tables.stage_times_alt
    } else {
        &tables.time_trial_times
    };
    let stage = record.record(&system.records().stage_tables)?.event_slot() as usize;
    if !table.with(|t| stage < t.stages.len())? {
        return Err(BadInput("a stage past its best times table"));
    }
    Ok((table, stage))
}

/// `f` on the chosen stage's ten best times; refused as `stage_times`.
pub(super) fn with_stage_times<R>(
    menus: &Menus,
    system: &System,
    tables: &BestTables,
    f: impl FnOnce(&mut rc2k_core::saves::entries::Times) -> R,
) -> Result<R, BadInput> {
    let (table, stage) = stage_times(menus, system, tables)?;
    table.with(|t| f(&mut t.stages[stage]))
}
