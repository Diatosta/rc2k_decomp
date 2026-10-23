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
//! 0x41CDBC (5). Key 0x26 is in `KEYS` twice (0x1D, then 0x14 never reached; kept,
//! section 16). Its calls through `part` (`CALLEES`); in the game their Rust versions, the
//! frame's run (`screens::run_frame`, the platform's) through the game. Ungated: the
//! Rust version passes the gate for each frame's keys and steps, not for the frame shown
//! (its function is gated) nor for the actions (some play or open screens, which gate
//! their own frames). Shadow mode runs the original alone. Self-test: a copy whose calls
//! go to stand-ins (the keys, the plays' ends and the actions scripted), against the Rust
//! version: each mode, keys of every kind, a camera edit, the developer's and the
//! editing keys, Escape twice, the leave switch.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, ZERO_FLAG, rva};
use crate::physics::flow::a;
use crate::race::viewer::{
    CAMERA_EDIT, CHOICE, CHOICE_BEFORE, CLOCK, DEVELOPER_KEYS, EDITING, ESCAPES, FRAME_CLOCK,
    HOLD_REVS, KEY_CLOCK, KEYS_IGNORED, LAST_RACE, LAST_RACE_VIEWED, LIST_LEAVE, LIST_SHOWN, MENU,
    MODE, MOVE_CLOCK, ONLY_SHOWN, Part, SLOT, SOUND, STOPPED, WAIT_RELEASE, gb, get, pb, put,
    regs_entry,
};

/// The calls by place, as IDA shows them.
pub const CALLEES: [usize; 18] = [
    0, 0x46F786, // 1 menu::replay_slot_loop (eax)
    0x41ECFF, // 2 race::save_last_replay (eax)
    0x41E9FF, // 3 race::open_viewed_replay (ebx; carry)
    0x41EF51, // 4 race::find_snapshot_number
    0x41CDC2, // 5 race::viewer_load_stage
    0x407F95, // 6 input::take_key
    0x41D456, // 7 race::mark_viewer_items (edi)
    0x41DE6F, // 8 race::viewer_action (ebx; ZF)
    0x405A71, // 9 screens::run_frame (eax the frame's function)
    0x41469A, // 10 menu::list_take_key (-> eax)
    0x41EC47, // 11 race::step_viewed_replay (carry)
    0x407897, // 12 gfx::take_screenshot
    0x414A55, // 13 menu::list_step (al, ebx, edi; ZF)
    0x414C1B, // 14 menu::list_item_unusable (edi; carry)
    0x40F059, // 15 sound::play_menu_sound (eax)
    0x41EB7C, // 16 race::close_viewed_replay
    0x41CEB4, // 17 race::viewer_tidy_stage
];
pub const FRAME_FUNCTION: usize = 0x41D5FB; // sub_41D5FB: race::viewer_frame
const QUITTING: usize = 0x51837C; // g_quitting
const SCREENSHOTS_ON: usize = 0x4F52B4; // g_screenshotsOn
const KEY_DOWN_A: usize = 0x4F6ADD; // byte_4F6ADD
const KEY_DOWN_B: usize = 0x4F6AF8; // byte_4F6AF8

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

/// The Rust version: the registers it leaves (the entry's).
pub fn replay_viewer(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    view(&mut r, part)?;
    // The end (0x41D436).
    part(16, &mut r)?;
    part(17, &mut r)?;
    put(STOPPED, 0);
    put(HOLD_REVS, 0);
    Ok(*entry)
}

/// Up to the end.
fn view(r: &mut Regs, part: Part) -> Result<(), Refused> {
    put(MODE, r.eax);
    put(SLOT, r.ebx);
    put(HOLD_REVS, 1);
    put(ONLY_SHOWN, 0);
    match r.eax {
        1 => {
            put(SLOT, LAST_RACE);
            put(STOPPED, 1);
        }
        2 | 3 => {
            let save = r.eax == 3;
            r.eax = save as u32;
            part(1, r)?;
            if r.eax == u32::MAX {
                return Ok(());
            }
            put(SLOT, r.eax);
            if save {
                part(2, r)?;
                return Ok(());
            }
            put(STOPPED, 1);
        }
        4 | 5 => {
            put(KEYS_IGNORED, (r.eax == 5) as u32);
            put(ONLY_SHOWN, 1);
            put(STOPPED, 0);
            put(SLOT, get(SLOT).wrapping_add(100));
        }
        _ => return Ok(()),
    }
    r.ebx = get(SLOT);
    part(3, r)?;
    if r.carry() {
        return Ok(());
    }
    put(CAMERA_EDIT, 0);
    r.eax = (get(SLOT) == LAST_RACE) as u32;
    put(LAST_RACE_VIEWED, r.eax);
    part(4, r)?;
    part(5, r)?;
    r.edi = a(MENU);
    put(SOUND, 0);
    pb(LIST_SHOWN, 1);
    pb(LIST_LEAVE, 0);
    r.eax = get(CLOCK);
    put(MOVE_CLOCK, r.eax);
    put(KEY_CLOCK, r.eax);
    put(CHOICE, 6);
    part(6, r)?;
    part(7, r)?;
    if get(ONLY_SHOWN) == 1 {
        r.ebx = 6;
        part(8, r)?;
        return Ok(());
    }
    loop {
        r.eax = a(FRAME_FUNCTION);
        part(9, r)?;
        r.eax = get(CLOCK);
        put(FRAME_CLOCK, r.eax);
        r.eax = get(CHOICE);
        put(CHOICE_BEFORE, r.eax);
        part(10, r)?;
        if get(QUITTING) == 1 {
            return Ok(());
        }
        if get(ONLY_SHOWN) == 1 {
            if get(STOPPED) == 1 {
                return Ok(());
            }
            if get(KEYS_IGNORED) == 1 {
                r.eax = 0;
            }
        }
        if get(WAIT_RELEASE) == 1 {
            if gb(KEY_DOWN_A) == 0 && gb(KEY_DOWN_B) == 0 {
                put(WAIT_RELEASE, 0);
            }
            r.eax = 0;
        }
        if r.eax as u8 == 1 {
            put(WAIT_RELEASE, 1);
            put(ESCAPES, get(ESCAPES).wrapping_add(1));
            if get(ESCAPES) == 2 {
                return Ok(());
            }
            r.eax = 0;
        }
        if r.eax as u8 != 0 {
            put(ESCAPES, 0);
        }
        if get(STOPPED) != 1 {
            part(11, r)?;
            if r.carry() || r.eax as u8 != 0 {
                put(STOPPED, 1);
            }
            continue;
        }
        let key = r.eax as u8;
        let next = if (key as i8) <= 0 {
            Next::Mark
        } else {
            stopped_key(r, part, key)?
        };
        let next = match next {
            Next::Action(action) => {
                r.ebx = action;
                part(8, r)?;
                if r.eflags & ZERO_FLAG != 0 {
                    Next::Mark
                } else {
                    Next::Sound
                }
            }
            other => other,
        };
        if let Next::Mark = next {
            mark(r, part, key)?;
        }
        // The menu sound asked for (0x41D41C).
        r.eax = get(SOUND);
        put(SOUND, 0);
        part(15, r)?;
        if gb(LIST_LEAVE) == 1 {
            return Ok(());
        }
    }
}

/// A key while stopped (0x41D139 ..): where it leads.
fn stopped_key(r: &mut Regs, part: Part, key: u8) -> Result<Next, Refused> {
    if get(SCREENSHOTS_ON) != 0 && key == 0x52 {
        part(12, r)?;
    }
    if get(CAMERA_EDIT) != 0 {
        return Ok(match key {
            0x68 => Next::Action(0x1E),
            0x69 => Next::Action(0x1F),
            1 => Next::Action(0x20),
            0x1C => Next::Action(0x21),
            _ => Next::Sound,
        });
    }
    r.esi = get(CHOICE)
        .wrapping_mul(0x10)
        .wrapping_add(r.edi)
        .wrapping_add(0x18);
    r.ebx = unsafe { ((r.edi + 0x10) as *const u32).read_unaligned() };
    part(13, r)?;
    if r.eflags & ZERO_FLAG != 0 {
        return Ok(Next::Mark);
    }
    if key == 0x1C {
        r.ebx = unsafe { ((r.esi + 1) as *const u8).read() } as u32;
        part(8, r)?;
        if r.eflags & ZERO_FLAG != 0 {
            return Ok(Next::Mark);
        }
    }
    if key == 0x3B {
        return Ok(Next::Action(6));
    }
    if key == 0x3C {
        return Ok(Next::Action(0x22));
    }
    if get(DEVELOPER_KEYS) == 1
        && let Some(&(_, action)) = DEVELOPER.iter().find(|&&(k, _)| k == key)
    {
        return Ok(Next::Action(action));
    }
    if get(EDITING) == 0 {
        return Ok(Next::Sound);
    }
    Ok(KEYS
        .iter()
        .find(|&&(k, _)| k == key)
        .map_or(Next::Sound, |&(_, action)| Next::Action(action)))
}

/// After a move or an action (0x41D3B0): the items marked again, the highlight put where
/// it can be chosen, the key's clock.
fn mark(r: &mut Regs, part: Part, key: u8) -> Result<(), Refused> {
    part(7, r)?;
    part(14, r)?;
    'tail: {
        if !r.carry() {
            break 'tail;
        }
        'reset: {
            if !MOVES.contains(&key) {
                break 'reset;
            }
            // On the way it went while it cannot be chosen.
            loop {
                r.ebx = unsafe { ((r.edi + 0x10) as *const u32).read_unaligned() };
                part(13, r)?;
                if r.eflags & ZERO_FLAG == 0 {
                    break;
                }
                part(14, r)?;
                if !r.carry() {
                    break 'tail;
                }
            }
            r.ebx = get(CHOICE_BEFORE);
            put(CHOICE, r.ebx);
            part(14, r)?;
            if !r.carry() {
                return Ok(());
            }
        }
        put(CHOICE, 6);
        if unsafe { ((r.edi + 0x78) as *const u8).read() } & 0xC0 != 0 {
            put(CHOICE, 0x13);
        }
    }
    if (key as i8) > 0 {
        r.ebx = get(FRAME_CLOCK);
        put(MOVE_CLOCK, r.ebx);
    }
    Ok(())
}

/// In the game: the Rust versions; the frame's run through the game (the platform's).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::*;
    unsafe {
        match place {
            1 => *r = crate::menu::replay_slot_loop::replay_slot_loop(r)?,
            2 => *r = save_last_replay::save_last_replay(r)?,
            3 => {
                *r = open_viewed_replay::open_viewed_replay(r, &mut open_viewed_replay::game_part)?
            }
            4 => find_snapshot_number::find_snapshot_number(),
            5 => *r = viewer_load_stage::viewer_load_stage(r)?,
            6 => viewer_load_stage::take_key(r),
            7 => mark_viewer_items::mark_viewer_items(r.edi),
            8 => *r = viewer_action::viewer_action(r, &mut viewer_action::game_part)?,
            9 => crate::calls::call(rva(CALLEES[9]), r),
            10 => *r = crate::menu::list_take_key::list_take_key(r)?,
            11 => *r = step_viewed_replay::step_viewed_replay(r)?,
            12 => crate::gfx::take_screenshot::take_screenshot()?,
            13 => *r = crate::menu::list_step::list_step(r),
            14 => {
                let unusable = crate::menu::list_item_unusable::list_item_unusable(r.edi);
                r.set_carry(unusable);
            }
            15 => crate::sound::play_menu_sound::play_menu_sound_in_game(r),
            16 => *r = close_viewed_replay::close_viewed_replay(r)?,
            _ => *r = viewer_tidy_stage::viewer_tidy_stage(r)?,
        }
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| replay_viewer(r, &mut game_part))
}
