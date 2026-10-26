//! `race::replay_viewer` (0x41CF0C): the replay viewer, by mode (eax):
//! 1 the last race's replay (222); 2 a saved one chosen (`menu::replay_slot_loop` 0; none:
//! nothing); 3 the last race's saved into a slot chosen (`replay_slot_loop` 1,
//! `race::save_last_replay`) and nothing else; 4 and 5 the game's own replay ebx (+100)
//! only shown, played at once (`race::viewer_action` 6), 5 with the keys ignored; other
//! modes nothing. `dword_5BEC9C` the mode, `dword_5BECA0` the replay, `g_holdCarRevs` 1
//! while it runs.
//!
//! Then the replay opened (`race::open_viewed_replay`; failing: the end), no camera edit,
//! `dword_5C2544` whether it is the last race's, the snapshots' number found
//! (`race::find_snapshot_number`), the stage loaded (`race::viewer_load_stage`), the list
//! set up (`unk_5C255C`: no sound, shown, not leaving, the clocks, item 6), the pending key
//! dropped, the items marked (`race::mark_viewer_items`). Each frame after (shown by
//! `screens::run_frame` with `sub_41D5FB`, `race::viewer_frame`): the clocks, a key
//! (`menu::list_take_key`); the game quitting, or (only shown) the play over, ends it; keys
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
//! In:  eax the mode, ebx the replay for modes 4 and 5
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::replay_mode_1`, `race::replay_mode_3`, `sub_41CD9D` (2), 0x41CDAF (4),
//! 0x41CDBC (5). Key 0x26 is in `KEYS` twice (0x1D, then 0x14 never reached; kept, section 16).

use crate::errors::Refused;
use crate::input::store::DEVICES;
use crate::install::vars::INSTALLER_PAGES;
use crate::menu::event_flow::Flow;
use crate::menu::screen_loop::Carried;
use crate::pointers::GamePtr;
use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::viewer::LAST_RACE;
use crate::race::viewer_screen_store::VIEWER_PANEL;
use crate::race::viewer_state_store::REPLAY_VIEW;
use crate::session::CLOCKS;
use crate::session::FLAGS;

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

/// Where the stopped viewer's key leads.
enum Next {
    Sound,
    Mark,
    Action(u32),
}

/// The Rust version: `mode` the viewer's mode (the original's eax), `c` the carried
/// values the slot screens are run with. A translated body: `r` is its scratch (the
/// original keeps every register of its caller).
pub fn replay_viewer(c: &Carried, mode: u32) -> Result<(), Refused> {
    let r = &mut Flow {
        c: Carried { arg: mode, ..*c },
        zero: false,
        record: 0,
    };
    view(r)?;
    // The end (0x41D436).
    step(16, r)?;
    step(17, r)?;
    REPLAY_FILE.viewer_stopped.set(0);
    REPLAY_FILE.hold_car_revs.set(0);
    Ok(())
}

/// Up to the end.
fn view(r: &mut Flow) -> Result<(), Refused> {
    REPLAY_FILE.viewer_mode.set(r.c.arg);
    REPLAY_FILE.viewer_slot.set(r.c.palette);
    REPLAY_FILE.hold_car_revs.set(1);
    REPLAY_VIEW.viewer_only_shown.set(0);
    match r.c.arg {
        1 => {
            REPLAY_FILE.viewer_slot.set(LAST_RACE);
            REPLAY_FILE.viewer_stopped.set(1);
        }
        2 | 3 => {
            let save = r.c.arg == 3;
            r.c.arg = save as u32;
            step(1, r)?;
            if r.c.arg == u32::MAX {
                return Ok(());
            }
            REPLAY_FILE.viewer_slot.set(r.c.arg);
            if save {
                step(2, r)?;
                return Ok(());
            }
            REPLAY_FILE.viewer_stopped.set(1);
        }
        4 | 5 => {
            REPLAY_VIEW.viewer_keys_ignored.set((r.c.arg == 5) as u32);
            REPLAY_VIEW.viewer_only_shown.set(1);
            REPLAY_FILE.viewer_stopped.set(0);
            REPLAY_FILE
                .viewer_slot
                .set(REPLAY_FILE.viewer_slot.get().wrapping_add(100));
        }
        _ => return Ok(()),
    }
    r.c.palette = REPLAY_FILE.viewer_slot.get();
    step(3, r)?;
    if r.c.carry {
        return Ok(());
    }
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .camera_edit
        .set(0);
    r.c.arg = (REPLAY_FILE.viewer_slot.get() == LAST_RACE) as u32;
    REPLAY_VIEW.last_race_viewed.set(r.c.arg);
    step(4, r)?;
    step(5, r)?;
    r.record = REPLAY_VIEW.viewer_list[0].addr();
    REPLAY_FILE.viewer_sound.set(0);
    INSTALLER_PAGES.list_answer.set(1);
    INSTALLER_PAGES.list_leave.set(0);
    r.c.arg = CLOCKS.key_ticks.get();
    INSTALLER_PAGES.move_clock.set(r.c.arg);
    INSTALLER_PAGES.key_clock.set(r.c.arg);
    INSTALLER_PAGES.choice.set(6);
    step(6, r)?;
    step(7, r)?;
    if REPLAY_VIEW.viewer_only_shown.get() == 1 {
        r.c.palette = 6;
        step(8, r)?;
        return Ok(());
    }
    loop {
        step(9, r)?;
        r.c.arg = CLOCKS.key_ticks.get();
        INSTALLER_PAGES.frame_clock.set(r.c.arg);
        r.c.arg = INSTALLER_PAGES.choice.get();
        INSTALLER_PAGES.choice_before.set(r.c.arg);
        step(10, r)?;
        if FLAGS.quitting.get() == 1 {
            return Ok(());
        }
        if REPLAY_VIEW.viewer_only_shown.get() == 1 {
            if REPLAY_FILE.viewer_stopped.get() == 1 {
                return Ok(());
            }
            if REPLAY_VIEW.viewer_keys_ignored.get() == 1 {
                r.c.arg = 0;
            }
        }
        if REPLAY_VIEW.wait_key_release.get() == 1 {
            if DEVICES.keys_down.get(1) == 0 && DEVICES.keys_down.get(0x1C) == 0 {
                REPLAY_VIEW.wait_key_release.set(0);
            }
            r.c.arg = 0;
        }
        if r.c.arg as u8 == 1 {
            REPLAY_VIEW.wait_key_release.set(1);
            REPLAY_VIEW
                .escape_count
                .set(REPLAY_VIEW.escape_count.get().wrapping_add(1));
            if REPLAY_VIEW.escape_count.get() == 2 {
                return Ok(());
            }
            r.c.arg = 0;
        }
        if r.c.arg as u8 != 0 {
            REPLAY_VIEW.escape_count.set(0);
        }
        if REPLAY_FILE.viewer_stopped.get() != 1 {
            step(11, r)?;
            if r.c.carry || r.c.arg as u8 != 0 {
                REPLAY_FILE.viewer_stopped.set(1);
            }
            continue;
        }
        let key = r.c.arg as u8;
        let next = if (key as i8) <= 0 {
            Next::Mark
        } else {
            stopped_key(r, key)?
        };
        let next = match next {
            Next::Action(action) => {
                r.c.palette = action;
                step(8, r)?;
                if r.zero { Next::Mark } else { Next::Sound }
            }
            other => other,
        };
        if let Next::Mark = next {
            mark(r, key)?;
        }
        // The menu sound asked for (0x41D41C).
        r.c.arg = REPLAY_FILE.viewer_sound.get();
        REPLAY_FILE.viewer_sound.set(0);
        step(15, r)?;
        if INSTALLER_PAGES.list_leave.get() == 1 {
            return Ok(());
        }
    }
}

/// A key while stopped (0x41D139 ..): where it leads.
fn stopped_key(r: &mut Flow, key: u8) -> Result<Next, Refused> {
    if crate::gfx::shots::on() != 0 && key == 0x52 {
        step(12, r)?;
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
    r.c.cars = INSTALLER_PAGES
        .choice
        .get()
        .wrapping_mul(0x10)
        .wrapping_add(r.record)
        .wrapping_add(0x18);
    r.c.palette = GamePtr::new(r.record + 0x10).read::<u32>("game data")?;
    step(13, r)?;
    if r.zero {
        return Ok(Next::Mark);
    }
    if key == 0x1C {
        r.c.palette = GamePtr::new(r.c.cars + 1).read::<u8>("game data")? as u32;
        step(8, r)?;
        if r.zero {
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
    if REPLAY_FILE.viewer_editing.get() == 0 {
        return Ok(Next::Sound);
    }
    Ok(KEYS
        .iter()
        .find(|&&(k, _)| k == key)
        .map_or(Next::Sound, |&(_, action)| Next::Action(action)))
}

/// After a move or an action (0x41D3B0): the items marked again, the highlight put where
/// it can be chosen, the key's clock.
fn mark(r: &mut Flow, key: u8) -> Result<(), Refused> {
    step(7, r)?;
    step(14, r)?;
    'tail: {
        if !r.c.carry {
            break 'tail;
        }
        'reset: {
            if !MOVES.contains(&key) {
                break 'reset;
            }
            // On the way it went while it cannot be chosen.
            loop {
                r.c.palette = GamePtr::new(r.record + 0x10).read::<u32>("game data")?;
                step(13, r)?;
                if !r.zero {
                    break;
                }
                step(14, r)?;
                if !r.c.carry {
                    break 'tail;
                }
            }
            r.c.palette = INSTALLER_PAGES.choice_before.get();
            INSTALLER_PAGES.choice.set(r.c.palette);
            step(14, r)?;
            if !r.c.carry {
                return Ok(());
            }
        }
        INSTALLER_PAGES.choice.set(6);
        if GamePtr::new(r.record + 0x78).read::<u8>("game data")? & 0xC0 != 0 {
            INSTALLER_PAGES.choice.set(0x13);
        }
    }
    if (key as i8) > 0 {
        r.c.palette = INSTALLER_PAGES.frame_clock.get();
        INSTALLER_PAGES.move_clock.set(r.c.palette);
    }
    Ok(())
}

/// The calls by place, as IDA shows them (the Rust versions; the frame's run through the
/// platform).
fn step(place: u32, r: &mut Flow) -> Result<(), Refused> {
    use crate::race::*;
    match place {
        1 => r.c.arg = crate::menu::replay_slot_loop::replay_slot_loop(&r.c)?,
        2 => save_last_replay::save_last_replay(r.c.arg)?,
        3 => r.c.carry = open_viewed_replay::open_viewed_replay(r.c.palette)?,
        4 => find_snapshot_number::find_snapshot_number()?,
        5 => viewer_load_stage::viewer_load_stage()?,
        6 => {
            // `input::take_key` into al and the zero flag.
            let key = crate::input::take_key::take_key();
            r.c.arg = r.c.arg & !0xFF | u32::from(key);
            r.zero = key == 0;
            r.c.carry = false;
        }
        7 => mark_viewer_items::mark_viewer_items(r.record)?,
        8 => r.zero = viewer_action::viewer_action(r.c.palette, &r.c)?,
        // `screens::run_frame` with `race::viewer_frame` (0x41D5FB).
        9 => {
            if let Some(lost) = crate::platform::run_frame_now(viewer_frame::viewer_frame) {
                lost?;
            }
        }
        10 => crate::menu::list_take_key::list_take_key().carry_on(r),
        11 => r.c.carry = step_viewed_replay::step_viewed_replay()?,
        12 => crate::gfx::take_screenshot::take_screenshot()?,
        13 => crate::menu::list_step::list_step(r.c.arg as u8, r.c.palette, r.record)?.carry_on(r),
        14 => r.c.carry = crate::menu::list_item_unusable::list_item_unusable(r.record)?,
        15 => crate::sound::play_menu_sound::play_menu_sound_in_game(r.c.arg),
        16 => close_viewed_replay::close_viewed_replay()?,
        _ => viewer_tidy_stage::viewer_tidy_stage()?,
    }
    Ok(())
}
