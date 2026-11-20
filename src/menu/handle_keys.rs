//! `menu::handle_keys`: takes one key and does what it means on a menu.
//!
//! Nothing happens (`Pressed::Nothing`) while an item is being edited (a service box turned
//! over) or the controller page is learning. Otherwise the menu sound and
//! `loop_flags.leave_screen` are cleared and the key is taken (`input::take_key`), first through
//! `menu::handle_yes_no`:
//! - none or 0x38: `Pressed::Nothing`;
//! - Escape (1): `loop_flags.leave_screen`, `draw_work.escape_pressed` and
//!   `captions.left_by_key` set; `Pressed::Escape`;
//! - Enter (0x1C): the Enter menu sound, the last-key time set to the clock, the Enter
//!   click played; `Pressed::Enter`;
//! - any other: the last-key time set, then the four `move_highlight_*` (0x66, 0x6B, 0x69,
//!   0x68), a screenshot (0x52), `loop_flags.toggle` flipped (0x20), 60 added to the service
//!   time budget when bit 14 of the cheat bits is set (0x14); `Pressed::Nothing`.
//!
//! Refuses, on a yes/no screen, a missing keyboard layout (before the key is taken).

use crate::errors::{BadInput, Refused};
use crate::input::key_to_char::has_layout;
use crate::input::take_key::take_key;
use crate::menu::handle_yes_no::handle_yes_no;
use crate::menu::navigation::{Direction, move_highlight};
use crate::owners::Owners;
use rc2k_core::menu::keys::Pressed;

/// `elapsed` is the time since the screen's set-up; its low byte is replaced by the key for the
/// yes / no screen.
pub fn handle_keys(o: Owners<'static>, elapsed: u32) -> Result<Pressed, Refused> {
    use rc2k_core::menu::keys::{self, Action, Move};
    let menus = o.menus;
    let nothing = Pressed::Nothing;
    if keys::keys_off(menus.screens.state.read().service_ui.box_shown, || {
        menus.screens.controls.learning.get()
    }) {
        return Ok(nothing);
    }
    if menus.screens.flow.read().asking && !has_layout(o.input) {
        return Err(BadInput("no keyboard layout").into());
    }
    menus.screens.entry.set_sound(keys::MenuSound::None);
    menus.screens.loop_flags.write().leave_screen = false;
    let key = take_key(o.input);
    if key == 0 {
        return Ok(nothing);
    }
    let key_over_elapsed = elapsed & !0xFF | key as u32;
    handle_yes_no(
        crate::strings::append_string_by_id::StringOwners::of(o),
        o.input,
        menus,
        key_over_elapsed,
    )?;
    let action = keys::action(key, || menus.play.cheats.read().cheats);
    match action {
        Action::Ignored => return Ok(nothing),
        Action::Escape => {
            menus.screens.loop_flags.write().leave_screen = true;
            menus.screens.draw_work.write().escape_pressed = true;
            menus.tables.captions.left_by_key.set(true);
            return Ok(Pressed::Escape);
        }
        _ => {}
    }
    let clock: u32 = o.system.clocks().frame.get();
    if action == Action::Enter {
        menus.screens.entry.set_sound(keys::MenuSound::Enter);
        menus.play.champ_view.write().key_clock.last_key_time = clock;
        crate::sound::play_menu_sound::play_menu_sound(
            o.sound,
            o.audio,
            o.menus,
            &o.system.session.picture_clock,
            keys::ENTER_CLICK,
        );
        return Ok(Pressed::Enter);
    }
    menus.play.champ_view.write().key_clock.last_key_time = clock;
    let direction = match action {
        Action::Move(Move::Up) => Direction::Up,
        Action::Move(Move::Down) => Direction::Down,
        Action::Move(Move::Right) => Direction::Right,
        Action::Move(Move::Left) => Direction::Left,
        Action::Screenshot => {
            crate::gfx::take_screenshot::take_screenshot(o.gfx, o.system)?;
            return Ok(nothing);
        }
        Action::Toggle => {
            let mut flags = menus.screens.loop_flags.write();
            flags.toggle = !flags.toggle;
            return Ok(nothing);
        }
        Action::AddTime => {
            menus.screens.event.write().service.minutes = menus
                .screens
                .event
                .read()
                .service
                .minutes
                .wrapping_add(keys::ADDED_TIME);
            return Ok(nothing);
        }
        _ => return Ok(nothing),
    };
    move_highlight(o, direction)?;
    Ok(nothing)
}
