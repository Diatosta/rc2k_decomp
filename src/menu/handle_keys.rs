//! `menu::handle_keys` (0x465FCC): takes one key and does what it means on a menu.
//!
//! In:  nothing; but bits 8 .. 15 of eax are read on Enter (see below)
//! Out: eax = 1 nothing to do for the caller, 2 Escape, 3 Enter; edx = the frame clock after
//! Enter and after any key not named here first; edi = `dword_731BBB` after one of the four moves;
//! everything else preserved Nothing happens (eax = 1) while an item is being edited
//! (`dword_75AA9C` = 1) or `dword_7551F4` is 1. Otherwise `dword_7556A4` = 0x6F, `dword_755561` =
//! 0, and the key: none        eax = 1 any         first `menu::handle_yes_no` 0x38        nothing
//! 1 (Escape)  `dword_755561`, `dword_755A3C` and `dword_758806` = 1; eax = 2 0x1C (Enter)
//! `dword_7556A4` = 0x14D, the last-key time `dword_7556C4` = the clock, `dword_7554D4` = 1 if ah
//! is 4 and else 0, the sound 2; eax = 3. `take_key` only changes al, so ah is what the caller had
//! in eax: reproduced as it is. others      the last-key time = the clock, then 0x66, 0x6B, 0x69,
//! 0x68  the four `move_highlight_*` 0x52      `sub_407897` (a screenshot, by what it calls) 0x20
//! flips `dword_755575` 0x14      adds 60 to the service time budget `dword_75593C`, if bit 14 of
//! `dword_75870C` is set The code after the Escape branch that could return 0 is never reached. 89
//! callers: every screen loop. Refuses, on a yes/no screen, a missing keyboard layout (before the
//! key is taken).

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

// ---- game data ----

// ---- Rust version ----

/// What `handle_keys` leaves for its caller (the original's eax, edx and edi).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Keys {
    /// 1 nothing to do, 2 Escape, 3 Enter.
    pub result: u32,
    /// The frame clock, after Enter and after any key not named first (edx).
    pub clock: Option<u32>,
    /// `dword_731BBB`, after one of the four moves (edi).
    pub menu: Option<crate::menu::menus_store::MenuId>,
}

/// `eax` is the caller's eax, whose bits 8 .. 15 are read on Enter (the screen loops: the
/// elapsed time).
pub fn handle_keys(eax: u32) -> Result<Keys, Refused> {
    // Converted: `rc2k_core::menu::keys` (`keys_off`, `action`, `entered_with_4`).
    use rc2k_core::menu::keys::{self, Action, Move};
    let nothing = Keys {
        result: keys::NOTHING,
        clock: None,
        menu: None,
    };
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
            return Ok(Keys {
                result: keys::ESCAPED,
                ..nothing
            });
        }
        _ => {}
    }
    let clock: u32 = CLOCKS.frame.get();
    if action == Action::Enter {
        OPTION_WORK.menu_sound.set(keys::ENTER_SOUND);
        CHAMP_VIEW.last_key_time.set(clock);
        SCREEN_FLOW.enter_with_4.set(keys::entered_with_4(eax));
        crate::sound::play_menu_sound::play_menu_sound_in_game(keys::ENTER_CLICK);
        return Ok(Keys {
            result: keys::ENTERED,
            clock: Some(clock),
            menu: None,
        });
    }
    CHAMP_VIEW.last_key_time.set(clock);
    let nothing = Keys {
        clock: Some(clock),
        ..nothing
    };
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
    let menu = crate::menu::menus_store::MenuId::from_code(
        crate::menu::texts_store::MENU_TEXTS.keys_menu.get(),
    )?;
    move_highlight(direction)?;
    Ok(Keys {
        menu: Some(menu),
        ..nothing
    })
}
