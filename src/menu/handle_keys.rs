//! `menu::handle_keys` (0x465FCC): takes one key and does what it means on a menu.
//!
//! Nothing happens (eax 1) while an item is being edited (`dword_75AA9C` 1) or
//! `dword_7551F4` is 1. Otherwise `dword_7556A4` = 0x6F, `dword_755561` = 0, and the key
//! taken (`input::take_key`, which changes only al), first through `menu::handle_yes_no`:
//! - none: eax 1;
//! - 0x38: nothing;
//! - 1 (Escape): `dword_755561`, `dword_755A3C` and `dword_758806` 1; eax 2;
//! - 0x1C (Enter): `dword_7556A4` 0x14D, the last-key time `dword_7556C4` the clock,
//!   `g_enterSkipsNameEntry` 1 if ah (what the caller had in eax) is 4, else 0, the sound
//!   2; eax 3;
//! - any other: the last-key time the clock, then 0x66, 0x6B, 0x69, 0x68 the four
//!   `move_highlight_*`, 0x52 a screenshot (`sub_407897`), 0x20 flips `dword_755575`,
//!   0x14 adds 60 to the service time budget `dword_75593C` if bit 14 of `dword_75870C` is
//!   set; eax 1.
//!
//! The code after the Escape branch that could return 0 is never reached.
//!
//! In:  bits 8 .. 15 of eax, read on Enter
//! Out: eax as above (edx the frame clock and edi `dword_731BBB` on some paths: nothing
//! reads them)
//!
//! 89 callers: every screen loop. Refuses, on a yes/no screen, a missing keyboard layout
//! (before the key is taken).

use crate::errors::{BadInput, Refused};
use crate::input::key_to_char::has_layout;
use crate::input::take_key::take_key;
use crate::menu::captions_store::CAPTIONS;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::cheats_store::CHEAT_VARS;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::handle_yes_no::handle_yes_no;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::navigation::{Direction, move_highlight};
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::state_store::MENU_STATE;
use crate::screens::controls_store::CONTROLS_SCREEN;
use crate::session::CLOCKS;
use rc2k_core::menu::keys::Pressed;

// ---- game data ----

// ---- Rust version ----

/// `eax` is the caller's eax, whose bits 8 .. 15 are read on Enter (the screen loops: the
/// elapsed time).
pub fn handle_keys(eax: u32) -> Result<Pressed, Refused> {
    // Converted: `rc2k_core::menu::keys` (`keys_off`, `action`, `enter_skips_name_entry`).
    use rc2k_core::menu::keys::{self, Action, Move};
    let nothing = Pressed::Nothing;
    if keys::keys_off(MENU_STATE.service_ui.box_shown.get(), || {
        CONTROLS_SCREEN.learning.get()
    }) {
        return Ok(nothing);
    }
    if SCREEN_FLOW.asking.get() == 1 && !has_layout() {
        return Err(BadInput("no keyboard layout").into());
    }
    OPTION_WORK.menu_sound.set(keys::NO_SOUND);
    LOOP_FLAGS.back.set(0u32);
    let key = take_key();
    if key == 0 {
        return Ok(nothing);
    }
    let eax = eax & !0xFF | key as u32;
    handle_yes_no(eax)?;
    let action = keys::action(key, || CHEAT_VARS.cheats.get());
    match action {
        Action::Ignored => return Ok(nothing),
        Action::Escape => {
            LOOP_FLAGS.back.set(1u32);
            DRAW_WORK.escaped.set(1u32);
            CAPTIONS.screen_left.set(1);
            return Ok(Pressed::Escape);
        }
        _ => {}
    }
    let clock: u32 = CLOCKS.frame.get();
    if action == Action::Enter {
        OPTION_WORK.menu_sound.set(keys::ENTER_SOUND);
        CHAMP_VIEW.last_key_time.set(clock);
        SCREEN_FLOW
            .enter_skips_name_entry
            .set(keys::enter_skips_name_entry(eax));
        crate::sound::play_menu_sound::play_menu_sound_in_game(keys::ENTER_CLICK);
        return Ok(Pressed::Enter);
    }
    CHAMP_VIEW.last_key_time.set(clock);
    let direction = match action {
        Action::Move(Move::Up) => Direction::Up,
        Action::Move(Move::Down) => Direction::Down,
        Action::Move(Move::Right) => Direction::Right,
        Action::Move(Move::Left) => Direction::Left,
        Action::Screenshot => {
            crate::gfx::take_screenshot::take_screenshot()?;
            return Ok(nothing);
        }
        Action::Toggle => {
            LOOP_FLAGS.toggle.set(LOOP_FLAGS.toggle.get() ^ 1);
            return Ok(nothing);
        }
        Action::AddTime => {
            EVENT_STATE.service_minutes.set(
                EVENT_STATE
                    .service_minutes
                    .get()
                    .wrapping_add(keys::ADDED_TIME),
            );
            return Ok(nothing);
        }
        _ => return Ok(nothing),
    };
    move_highlight(direction)?;
    Ok(nothing)
}
