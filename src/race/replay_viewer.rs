//! `race::replay_viewer` (0x41CF0C): the replay viewer, by mode (eax, `Mode`):
//! 1 the last race's replay (222); 2 a saved one chosen (`menu::replay_slot_loop` 0; none:
//! nothing); 3 the last race's saved into a slot chosen (`replay_slot_loop` 1,
//! `race::save_last_replay`) and nothing else. `dword_5BEC9C` the mode, `dword_5BECA0` the
//! replay, `g_holdCarRevs` 1 while it runs. (Modes 4 and 5, the game's own replay ebx + 100
//! only shown, 5 with the keys ignored, are entered only at 0x41CDA8 and 0x41CDB5, which
//! nothing references: not kept.)
//!
//! Then the replay opened (`race::open_viewed_replay`; failing: the end), no camera edit,
//! `dword_5C2544` whether it is the last race's, the snapshots' number found
//! (`race::find_snapshot_number`), the stage loaded (`race::viewer_load_stage`), the list
//! set up (`unk_5C255C`: no sound, shown, not leaving, the clocks, item 6), the pending key
//! dropped, the items marked (`race::mark_viewer_items`). Each frame after (shown by
//! `screens::run_frame` with `sub_41D5FB`, `race::viewer_frame`): the clocks, a key
//! (`menu::list_take_key`); the game quitting ends it; keys
//! held after a play (`dword_5C2554`, until `byte_4F6ADD` and `byte_4F6AF8` are up) do
//! nothing; Escape twice in a row ends it. Playing (`dword_5BEC98` 0): a frame on
//! (`race::step_viewed_replay`); its end or any key stops. Stopped: key 0x52 (Numpad 0) a screenshot
//! (`gfx::take_screenshot`, with screenshots on); while a camera is edited 0x68, 0x69,
//! Escape, Enter are actions 0x1E, 0x1F, 0x20, 0x21; else the arrows move the highlight
//! (`menu::list_step`), Enter does the item's action (its byte 1), 0x3B and 0x3C actions 6
//! and 0x22, the developer's keys (`dword_5C2CEB`) '?', '@', 'A', 'B' 0x26, 0x27, 0x28,
//! 0x25, and with editing (`dword_5BECA8`) the keys of `KEYS`. After an action or a move
//! the items are marked again and a highlight that cannot be chosen
//! (`menu::list_item_unusable`) moved on the way it went, else back, else to item 6 (or
//! 0x13 when item 6 is marked); the menu sound asked (`dword_5BECA4`) played
//! (`sound::play_menu_sound`); `byte_51D416` 1 ends it. At the end the replay closed
//! (`race::close_viewed_replay`), the front end back (`race::viewer_tidy_stage`),
//! `dword_5BEC98` and `g_holdCarRevs` 0.
//!
//! In:  eax the mode
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::replay_mode_1`, `race::replay_mode_3`, `race::replay_mode_2`. Key 0x26
//! is in `KEYS` twice (0x1D, then 0x14 never reached; kept, section 16).

use crate::errors::Refused;
use crate::input::owner::input;
use crate::menu::list_handle::ListMenu;
use crate::race::owner::Race;
use crate::race::viewer::LAST_RACE;
use crate::race::viewer_screen_store::VIEWER_PANEL;
use crate::race::viewer_state_store::ReplayViewBlock;

/// (key, action) with editing on, in the original's order (0x26 twice: the first wins).
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

/// What the viewer is started for (the original's eax).
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

/// The Rust version.
pub fn replay_viewer(race: &Race, mode: Mode) -> Result<(), Refused> {
    view(race, mode)?;
    // The end (0x41D436).
    crate::race::close_viewed_replay::close_viewed_replay(race)?;
    crate::race::viewer_tidy_stage::viewer_tidy_stage(race)?;
    race.shared.replay_file.viewer_stopped.set(0);
    race.shared.replay_file.hold_car_revs.set(0);
    Ok(())
}

/// Up to the end.
fn view(race: &Race, mode: Mode) -> Result<(), Refused> {
    let system = crate::system::owner::system();
    let devices = &input().devices;
    let session = &system.session;
    let replay = &race.replay;
    use crate::race::*;
    race.shared.replay_file.viewer_mode.set(mode as u32);
    race.shared.replay_file.hold_car_revs.set(1);
    replay.replay_view.viewer_only_shown.set(0);
    let slot = match mode {
        Mode::LastRace => LAST_RACE,
        Mode::Saved | Mode::SaveLast => {
            let saving = mode == Mode::SaveLast;
            let slot = crate::menu::replay_slot_loop::replay_slot_loop(
                crate::menu::owner::menus(),
                saving,
            )?;
            if slot == u32::MAX {
                return Ok(());
            }
            if saving {
                race.shared.replay_file.viewer_slot.set(slot);
                return save_last_replay::save_last_replay(race, slot);
            }
            slot
        }
    };
    race.shared.replay_file.viewer_slot.set(slot);
    race.shared.replay_file.viewer_stopped.set(1);
    let failed = open_viewed_replay::open_viewed_replay(race, slot)?;
    if failed {
        return Ok(());
    }
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .camera_edit
        .set(0);
    replay
        .replay_view
        .last_race_viewed
        .set((slot == LAST_RACE) as u32);
    find_snapshot_number::find_snapshot_number(&race.replay)?;
    viewer_load_stage::viewer_load_stage(race)?;
    let list = ListMenu::of(crate::pointers::GamePtr::to(
        &replay.replay_view.viewer_list,
    ));
    race.shared.replay_file.viewer_sound.set(0);
    crate::install::owner::install().pages.list_answer.set(1);
    crate::install::owner::install().pages.list_leave.set(0);
    let now = session.clocks.key_ticks.get();
    crate::install::owner::install().pages.move_clock.set(now);
    crate::install::owner::install().pages.key_clock.set(now);
    crate::install::owner::install().pages.choice.set(6);
    // The pending key dropped.
    crate::input::take_key::take_key();
    mark_viewer_items::mark_viewer_items(race, list)?;
    // A test run's picture check: the item chosen (play) at once, as Return would.
    #[cfg(feature = "remote")]
    if crate::remote::shots_on() {
        let chosen = list.item(crate::install::owner::install().pages.choice.get());
        action_next(race, u32::from(chosen.viewer_action()))?;
    }
    loop {
        // `screens::run_frame` with `race::viewer_frame` (0x41D5FB).
        if let Some(lost) = crate::platform::run_frame_now(viewer_frame::viewer_frame) {
            lost?;
        }
        crate::install::owner::install()
            .pages
            .frame_clock
            .set(session.clocks.key_ticks.get());
        crate::install::owner::install()
            .pages
            .choice_before
            .set(crate::install::owner::install().pages.choice.get());
        let mut key = crate::menu::list_take_key::list_take_key();
        if session.flags.quitting.get() == 1 {
            return Ok(());
        }
        if replay.replay_view.wait_key_release.get() == 1 {
            if devices.keys_down.get(1) == 0 && devices.keys_down.get(0x1C) == 0 {
                replay.replay_view.wait_key_release.set(0);
            }
            key = 0;
        }
        if key == 1 {
            replay.replay_view.wait_key_release.set(1);
            replay
                .replay_view
                .escape_count
                .set(replay.replay_view.escape_count.get().wrapping_add(1));
            if replay.replay_view.escape_count.get() == 2 {
                return Ok(());
            }
            key = 0;
        }
        if key != 0 {
            replay.replay_view.escape_count.set(0);
        }
        if race.shared.replay_file.viewer_stopped.get() != 1 {
            let ended = step_viewed_replay::step_viewed_replay(race)?;
            if ended || key != 0 {
                race.shared.replay_file.viewer_stopped.set(1);
            }
            continue;
        }
        let next = if (key as i8) <= 0 {
            Next::Mark
        } else {
            stopped_key(race, list, key)?
        };
        let next = match next {
            Next::Action(action) => action_next(race, action)?,
            other => other,
        };
        if let Next::Mark = next {
            mark(race, list, key)?;
        }
        // The menu sound asked for (0x41D41C).
        let sound = race.shared.replay_file.viewer_sound.get();
        race.shared.replay_file.viewer_sound.set(0);
        crate::sound::play_menu_sound::play_menu_sound_in_game(sound);
        if crate::install::owner::install().pages.list_leave.get() == 1 {
            return Ok(());
        }
    }
}

/// An action done: the items are marked again after one that is an action.
fn action_next(race: &Race, action: u32) -> Result<Next, Refused> {
    Ok(
        if crate::race::viewer_action::viewer_action(race, action)? {
            Next::Mark
        } else {
            Next::Sound
        },
    )
}

/// A key while stopped (0x41D139 ..): where it leads.
fn stopped_key(race: &Race, list: ListMenu, key: u8) -> Result<Next, Refused> {
    if crate::gfx::shots::on() != 0 && key == 0x52 {
        crate::gfx::take_screenshot::take_screenshot()?;
    }
    if crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .camera_edit
        .get()
        != 0
    {
        return Ok(match key {
            0x68 => Next::Action(0x1E),
            0x69 => Next::Action(0x1F),
            1 => Next::Action(0x20),
            0x1C => Next::Action(0x21),
            _ => Next::Sound,
        });
    }
    if crate::menu::list_step::list_step(key, Some(ReplayViewBlock::key_table()?), list)? {
        return Ok(Next::Mark);
    }
    if key == 0x1C {
        let chosen = list.item(crate::install::owner::install().pages.choice.get());
        if let Next::Mark = action_next(race, u32::from(chosen.viewer_action()))? {
            return Ok(Next::Mark);
        }
    }
    if key == 0x3B {
        return Ok(Next::Action(6));
    }
    if key == 0x3C {
        return Ok(Next::Action(0x22));
    }
    if VIEWER_PANEL.developer_keys() == 1
        && let Some(&(_, action)) = DEVELOPER.iter().find(|&&(k, _)| k == key)
    {
        return Ok(Next::Action(action));
    }
    if race.shared.replay_file.viewer_editing.get() == 0 {
        return Ok(Next::Sound);
    }
    Ok(KEYS
        .iter()
        .find(|&&(k, _)| k == key)
        .map_or(Next::Sound, |&(_, action)| Next::Action(action)))
}

/// After a move or an action (0x41D3B0): the items marked again, the highlight put where
/// it can be chosen, the key's clock.
fn mark(race: &Race, list: ListMenu, key: u8) -> Result<(), Refused> {
    use crate::menu::list_item_unusable::list_item_unusable;
    crate::race::mark_viewer_items::mark_viewer_items(race, list)?;
    'tail: {
        if !list_item_unusable(list) {
            break 'tail;
        }
        'reset: {
            if !MOVES.contains(&key) {
                break 'reset;
            }
            // On the way it went while it cannot be chosen.
            loop {
                if !crate::menu::list_step::list_step(
                    key,
                    Some(ReplayViewBlock::key_table()?),
                    list,
                )? {
                    break;
                }
                if !list_item_unusable(list) {
                    break 'tail;
                }
            }
            // Else back where it was.
            crate::install::owner::install()
                .pages
                .choice
                .set(crate::install::owner::install().pages.choice_before.get());
            if !list_item_unusable(list) {
                return Ok(());
            }
        }
        // Else item 6, or 0x13 when item 6 is marked.
        crate::install::owner::install().pages.choice.set(6);
        if list.item(6).first() & 0xC0 != 0 {
            crate::install::owner::install().pages.choice.set(0x13);
        }
    }
    if (key as i8) > 0 {
        crate::install::owner::install()
            .pages
            .move_clock
            .set(crate::install::owner::install().pages.frame_clock.get());
    }
    Ok(())
}
