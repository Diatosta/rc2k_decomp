//! `menu::replay_viewer`: the replay viewer, by mode: 1 the last race's replay (222); 2
//! a saved one chosen (`menu::replay_slot_loop` 0; none: nothing); 3 the last race's saved into a
//! slot chosen (`replay_slot_loop` 1, `race::save_last_replay`) and nothing else. `viewer_mode`
//! the mode, `viewer_slot` the replay, `hold_car_revs` 1 while it runs.
//!
//! Then the replay is opened (`race::open_viewed_replay`; failing: the end), with no camera edit,
//! `last_race_viewed` whether it is the last race's, the snapshots' number found
//! (`race::find_snapshot_number`), the stage loaded (`race::viewer_load_stage`), the list set up
//! (`viewer_list`: no sound, shown, not leaving, the clocks, item 6), the pending key dropped and
//! the items marked (`menu::replay_viewer::mark_items`). Each frame after (shown by `screens::run_frame`
//! with `menu::replay_viewer::frame`): the clocks, a key (`menu::list_take_key`); the game quitting ends
//! it; keys held after a play (`wait_key_release`, until `Devices.keys_down` and
//! `Devices.keys_down` are up) do nothing; Escape twice in a row ends it. Playing (`viewer_stopped`
//! 0): a frame on (`race::step_viewed_replay`); its end or any key stops. Stopped: key 0x52 (Numpad
//! 0) a screenshot (`gfx::take_screenshot`, with screenshots on); while a camera is edited 0x68,
//! 0x69, Escape, Enter are actions 0x1E, 0x1F, 0x20, 0x21; else the arrows move the highlight
//! (`menu::replay_viewer::list_step`), Enter does the item's action (its byte 1), 0x3B and 0x3C actions 6 and
//! 0x22, the developer's keys (`ViewerAndKinds.viewer_panel.untyped_tail[0]`) '?', '@', 'A', 'B'
//! 0x26, 0x27, 0x28, 0x25, and with editing (`viewer_editing`) the keys of `KEYS`. After an action
//! or a move the items are marked again and a highlight that cannot be chosen
//! (`menu::replay_viewer::list_item_unusable`) moved on the way it went, else back, else to item 6 (or 0x13 when
//! item 6 is marked); the menu sound asked (`viewer_sound`) played (`sound::play_menu_sound`);
//! `list_leave` 1 ends it. At the end the replay closed (`race::close_viewed_replay`), the front
//! end back (`race::viewer_tidy_stage`), `viewer_stopped` and `hold_car_revs` 0.
//!
//! A replay named when the program was started (`launched_replay`) is viewed as mode 2's, its
//! slot given instead of chosen on the loading screen, and it plays at once, as Enter on the
//! item the viewer opens on plays it.
//!
//! Takes the mode.
//!
//! Key 0x26 is in `KEYS` twice (0x1D, then 0x14 never reached; a slip kept as is).

pub(crate) mod action;
mod frame;
mod item;
mod list_clear_marks;
mod list_item_unusable;
mod list_step;
mod mark_items;
mod panel;
pub(crate) mod render_dialog;
mod render_progress;
pub(crate) mod render_row;
pub(crate) mod replay_mode;
pub(crate) mod replay_mode_2;
mod tip;

use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::owner::Race;
use crate::race::viewer::LAST_RACE;
use crate::race::viewer_list::ViewerList;
use crate::race::viewer_state_store::ReplayViewBlock;

/// (key, action) with editing on, in order (0x26 twice: the first wins).
pub const KEYS: [(u8, u32); 30] = [
    (0x39, 6),
    (0x64, 0x1A),
    (0x35, 0x23),
    (0x2C, 0x13),
    (0x2D, 0x11),
    (0x2E, 0x1B),
    (0x2F, 0x12),
    (0x1E, 2),
    (0x30, 0xA),
    (0x24, 0x24),
    (0x25, 0x1C),
    (0x26, 0x1D),
    (0x18, 0xE),
    (0x19, 0x18),
    (0x31, 0xF),
    (0x32, 0x10),
    (0x26, 0x14),
    (0x1F, 0x16),
    (0x0C, 5),
    (0x0D, 7),
    (0x33, 4),
    (0x34, 8),
    (0x1A, 2),
    (0x1B, 0xA),
    (0x6D, 0x12),
    (0x6E, 0x11),
    (0x65, 0),
    (0x6A, 0xC),
    (0x67, 9),
    (0x6C, 3),
];
const DEVELOPER: [(u8, u32); 4] = [(b'?', 0x26), (b'@', 0x27), (b'A', 0x28), (b'B', 0x25)];
const MOVES: [u8; 4] = [0x66, 0x6B, 0x68, 0x69];

/// What the viewer is started for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// The last race's replay.
    LastRace = 1,
    /// A saved replay, chosen on the loading screen.
    Saved = 2,
    /// The last race's replay saved into a slot chosen, and nothing else.
    SaveLast = 3,
}

/// Where the stopped viewer's key leads.
enum Next {
    Sound,
    Mark,
    Action(u32),
}

/// How the viewer comes by its replay and what it does with it first.
#[derive(Clone, Copy)]
enum Start {
    /// From a menu: the mode's replay, the viewer stopped until a key plays it.
    Menu(Mode),
    /// From the program's start: the saved replay `slot`, played at once; `leave_when_played`
    /// ends the viewer when that play stops.
    Launched { slot: u32, leave_when_played: bool },
}

/// How a view ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Viewed {
    /// No replay was opened: none chosen, or its file could not be read.
    Nothing,
    /// A replay was to be saved into a slot and could not be (its recording cannot be read, or
    /// the file cannot be written).
    NotSaved,
    /// The viewer ran and was left.
    Left,
    /// The viewer ran and its first play stopped (a launch that leaves then).
    Played,
}

pub fn replay_viewer(
    race: &'static Race,
    o: Owners<'static>,
    mode: Mode,
) -> Result<Viewed, Refused> {
    let viewed = viewing(race, o, Start::Menu(mode))?;
    end(race, o)?;
    Ok(viewed)
}

/// The viewer on the saved replay `slot`, as mode 2 runs it on the slot its loading screen
/// leaves, playing at once; with `leave_when_played` it ends when that play stops (the replay's
/// end or a key) and the front end is not brought back: the caller ends the program.
pub fn launched_replay(
    race: &'static Race,
    o: Owners<'static>,
    slot: u32,
    leave_when_played: bool,
) -> Result<Viewed, Refused> {
    let start = Start::Launched {
        slot,
        leave_when_played,
    };
    let viewed = viewing(race, o, start)?;
    if viewed == Viewed::Played {
        crate::race::close_viewed_replay::close_viewed_replay(race)?;
    } else {
        end(race, o)?;
    }
    Ok(viewed)
}

/// `view`, with a trouble the game can go back from (a replay's data that cannot be used)
/// ending the viewer as leaving it does, then on up to the main menu.
fn viewing(race: &'static Race, o: Owners<'static>, start: Start) -> Result<Viewed, Refused> {
    // Data the viewer refuses while a replay plays is the replay's: a file that opened and is
    // damaged further in sends many different checks off, so any refusal here is the file's.
    let guarded = crate::race::dialogs::guarded(|| {
        view(race, o, start).map_err(|why| match why {
            Refused::BadInput(crate::errors::BadInput(reason)) => {
                crate::errors::Recover(reason).into()
            }
            other => other,
        })
    });
    match guarded {
        Ok(viewed) => viewed,
        Err(reason) => {
            if end(race, o).is_err() {
                crate::program_log::log("recover: the viewer could not be closed");
            }
            crate::race::dialogs::back(&reason)
        }
    }
}

/// The viewer's end: the replay closed, the front end back.
fn end(race: &'static Race, o: Owners<'static>) -> Result<(), Refused> {
    crate::race::close_viewed_replay::close_viewed_replay(race)?;
    crate::race::viewer_tidy_stage::viewer_tidy_stage(race, o)?;
    race.shared.replay_file().viewer_stopped.set(false);
    race.shared.replay_file().hold_car_revs.set(0);
    Ok(())
}

/// Up to the end.
fn view(race: &'static Race, o: Owners<'static>, start: Start) -> Result<Viewed, Refused> {
    let (system, list_state) = (o.system, &o.menus.list);
    let devices = &o.input.devices;
    let session = &system.session;
    let replay = &race.replay;
    use crate::race::{
        find_snapshot_number, open_viewed_replay, save_last_replay, step_viewed_replay,
        viewer_load_stage,
    };
    let (mode, launched) = match start {
        Start::Menu(mode) => (mode, None),
        Start::Launched { slot, .. } => (Mode::Saved, Some(slot)),
    };
    let leave_when_played = matches!(
        start,
        Start::Launched {
            leave_when_played: true,
            ..
        }
    );
    crate::race::render_row::select(race, false);
    crate::race::viewer_tip::reset(race);
    race.shared.replay_file().viewer_mode.set(mode as u32);
    race.shared.replay_file().hold_car_revs.set(1);
    replay.replay_view().viewer_only_shown.set(0);
    let slot = match (mode, launched) {
        (_, Some(slot)) => slot,
        (Mode::LastRace, None) => LAST_RACE,
        (Mode::Saved | Mode::SaveLast, None) => {
            let saving = mode == Mode::SaveLast;
            let slot = crate::menu::replay_slot_loop::replay_slot_loop(o, saving)?;
            if slot == u32::MAX {
                return Ok(Viewed::Nothing);
            }
            if saving {
                race.shared.replay_file().viewer_slot.set(slot);
                let saved = save_last_replay::save_last_replay(race, o.system, slot)?;
                return Ok(if saved {
                    Viewed::Nothing
                } else {
                    Viewed::NotSaved
                });
            }
            slot
        }
    };
    race.shared.replay_file().viewer_slot.set(slot);
    race.shared.replay_file().viewer_stopped.set(true);
    let opened = open_viewed_replay::open_viewed_replay(race, o.system, slot)?;
    if opened.failed() {
        if opened == open_viewed_replay::Opened::Damaged {
            tell_damaged(slot);
        }
        return Ok(Viewed::Nothing);
    }
    race.viewer_buffers().camera_edit.set(0);
    replay.replay_view().last_race_viewed.set(slot == LAST_RACE);
    find_snapshot_number::find_snapshot_number(&race.replay)?;
    viewer_load_stage::viewer_load_stage(race, o)?;
    let list = &o.race.replay.replay_view().viewer_list;
    race.shared.replay_file().viewer_sound.set(0);
    list_state.list_answer.set(1);
    list_state.list_leave.set(0);
    let now = session.key_clock.ticks.get();
    list_state.move_clock.set(now);
    list_state.key_clock.set(now);
    list_state.choice.set(6);
    // The pending key dropped.
    crate::input::take_key::take_key(o.input);
    mark_items::mark_viewer_items(race, list)?;
    // A launched replay, and a test run's picture check: the item chosen (play) at once, as
    // Return would.
    let play_at_once = launched.is_some() || crate::remote::shots_on();
    if play_at_once {
        if crate::video::active() {
            // A render from the command line plays the part it was given.
            let (from, to) = crate::video::range().unwrap_or((0, None));
            crate::race::render_row::play_range(race, from, to, o)?;
        } else {
            action_next(race, o, list.action(list_state.choice.get()))?;
        }
        // The play runs to its stop inside the action.
        if leave_when_played {
            return Ok(Viewed::Played);
        }
    }
    loop {
        // `screens::run_frame` with `menu::replay_viewer::frame`.
        if let Some(lost) = crate::shell::frame::run_frame_now(o, || frame::viewer_frame(o)) {
            lost?;
        }
        list_state.frame_clock.set(session.key_clock.ticks.get());
        list_state.choice_before.set(list_state.choice.get());
        let mut key = crate::menu::list_take_key::list_take_key(list_state, o.input);
        if session.flags.quitting.is_set() {
            return Ok(Viewed::Left);
        }
        if replay.replay_view().wait_key_release.get() == 1 {
            if devices.keys_down.get(1) == 0 && devices.keys_down.get(0x1C) == 0 {
                replay.replay_view().wait_key_release.set(0);
            }
            key = 0;
        }
        if key == 1 {
            replay.replay_view().wait_key_release.set(1);
            replay
                .replay_view()
                .escape_count
                .set(replay.replay_view().escape_count.get().wrapping_add(1));
            if replay.replay_view().escape_count.get() == 2 {
                return Ok(Viewed::Left);
            }
            key = 0;
        }
        if key != 0 {
            replay.replay_view().escape_count.set(0);
        }
        if !race.shared.replay_file().viewer_stopped.is_set() {
            let ended = step_viewed_replay::step_viewed_replay(race, system)?;
            if ended || key != 0 {
                race.shared.replay_file().viewer_stopped.set(true);
            }
            continue;
        }
        let next = if (key as i8) <= 0 {
            Next::Mark
        } else {
            stopped_key(race, o, list, key)?
        };
        let next = match next {
            Next::Action(action) => action_next(race, o, action)?,
            other => other,
        };
        if let Next::Mark = next {
            mark(race, list_state, list, key)?;
        }
        // The menu sound asked for.
        let sound = race.shared.replay_file().viewer_sound.get();
        race.shared.replay_file().viewer_sound.set(0);
        crate::sound::play_menu_sound::play_menu_sound(
            o.sound,
            o.audio,
            o.menus,
            &o.system.session.picture_clock,
            sound,
        );
        if list_state.list_leave.get() == 1 {
            return Ok(Viewed::Left);
        }
    }
}

/// An action done: the items are marked again after one that is an action.
fn action_next(race: &'static Race, o: Owners<'static>, action: u32) -> Result<Next, Refused> {
    Ok(
        if crate::menu::replay_viewer::action::viewer_action(race, o, action)? {
            Next::Mark
        } else {
            Next::Sound
        },
    )
}

/// A key while stopped: where it leads.
fn stopped_key(
    race: &'static Race,
    o: Owners<'static>,
    list: &ViewerList,
    key: u8,
) -> Result<Next, Refused> {
    let list_state = &o.menus.list;
    if o.gfx.shots.on.get() != 0 && key == 0x52 {
        crate::gfx::take_screenshot::take_screenshot(o.gfx, o.system)?;
    }
    if race.viewer_buffers().camera_edit.get() != 0 {
        return Ok(match key {
            0x68 => Next::Action(0x1E),
            0x69 => Next::Action(0x1F),
            1 => Next::Action(0x20),
            0x1C => Next::Action(0x21),
            _ => Next::Sound,
        });
    }
    // The "render to video" row under the buttons: the down arrow from the lower row reaches
    // it, up or left leaves it, Enter renders.
    use crate::menu::replay_viewer::render_row::{
        KEY_DOWN, KEY_ENTER, KEY_LEFT, KEY_UP, LOWER_ROW,
    };
    if crate::race::render_row::selected(race) {
        return Ok(match key {
            KEY_UP | KEY_LEFT => {
                crate::race::render_row::select(race, false);
                Next::Mark
            }
            KEY_ENTER => {
                crate::race::render_row::select(race, false);
                crate::menu::replay_viewer::render_row::render(race, o)?;
                Next::Mark
            }
            _ => Next::Sound,
        });
    }
    // Down from the lower row, or from an upper-row button whose way down is not open (the table
    // has none, or the button under it is greyed): the render button, before the list moves the
    // highlight sideways to some other button.
    if key == KEY_DOWN
        && crate::menu::replay_viewer::render_row::OFFERED
        && race.shared.replay_file().index.block_count.get() != 0
        && (LOWER_ROW.contains(&list_state.choice.get())
            || way_down_closed(list, list_state.choice.get())?)
    {
        crate::race::render_row::select(race, true);
        return Ok(Next::Mark);
    }
    let stepped = crate::menu::replay_viewer::list_step::list_step(
        list_state,
        key,
        Some(ReplayViewBlock::key_table()),
    )?;
    if stepped {
        return Ok(Next::Mark);
    }
    if key == 0x1C
        && let Next::Mark = action_next(race, o, list.action(list_state.choice.get()))?
    {
        return Ok(Next::Mark);
    }
    if key == 0x3B {
        return Ok(Next::Action(6));
    }
    if key == 0x3C {
        return Ok(Next::Action(0x22));
    }
    if race.viewer_panel().developer_keys() == 1
        && let Some(&(_, action)) = DEVELOPER.iter().find(|&&(k, _)| k == key)
    {
        return Ok(Next::Action(action));
    }
    if race.shared.replay_file().viewer_editing.get() == 0 {
        return Ok(Next::Sound);
    }
    Ok(KEYS
        .iter()
        .find(|&&(k, _)| k == key)
        .map_or(Next::Sound, |&(_, action)| Next::Action(action)))
}

/// Whether the down arrow from the upper-row item `from` leads nowhere: the key table has no
/// way down from it, or the item it leads to is greyed.
fn way_down_closed(list: &ViewerList, from: u32) -> Result<bool, Refused> {
    use crate::menu::replay_viewer::render_row::LOWER_ROW;
    if from >= *LOWER_ROW.start() {
        return Ok(false);
    }
    let entry = ReplayViewBlock::key_table()
        .add(from.wrapping_mul(4) + 1)
        .read(1, "the viewer's key table")?[0];
    Ok((entry as i8) < 0
        || u32::from(entry) >= crate::race::viewer_list::ITEMS
        || rc2k_core::menu::list_menu::unusable(list.item(u32::from(entry)).first.get()))
}

/// After a move or an action: the items marked again, the highlight put where
/// it can be chosen, the key's clock.
fn mark(
    race: &Race,
    list_state: &crate::menu::list_state::ListState,
    list: &ViewerList,
    key: u8,
) -> Result<(), Refused> {
    use crate::menu::replay_viewer::list_item_unusable::list_item_unusable;
    crate::menu::replay_viewer::mark_items::mark_viewer_items(race, list)?;
    'tail: {
        if !list_item_unusable(list_state, list) {
            break 'tail;
        }
        'reset: {
            if !MOVES.contains(&key) {
                break 'reset;
            }
            // On the way it went while it cannot be chosen.
            loop {
                if !crate::menu::replay_viewer::list_step::list_step(
                    list_state,
                    key,
                    Some(ReplayViewBlock::key_table()),
                )? {
                    break;
                }
                if !list_item_unusable(list_state, list) {
                    break 'tail;
                }
            }
            // Else back where it was.
            list_state.choice.set(list_state.choice_before.get());
            if !list_item_unusable(list_state, list) {
                return Ok(());
            }
        }
        // Else item 6, or 0x13 when item 6 is marked.
        list_state.choice.set(6);
        if list.item(6).first.get() & 0xC0 != 0 {
            list_state.choice.set(0x13);
        }
    }
    if (key as i8) > 0 {
        list_state.move_clock.set(list_state.frame_clock.get());
    }
    Ok(())
}

/// A replay that could not be opened is said so: it is skipped and the player stays in the
/// menu. The last race's slot is left out: no race may have been recorded.
pub(super) fn tell_damaged(slot: u32) {
    if slot < 100 {
        crate::race::dialogs::notice(
            rc2k_core::strings::own::id::DAMAGED_REPLAY,
            &(slot + 1).to_string(),
        );
    }
}
