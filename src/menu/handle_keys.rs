//! `menu::handle_keys`: takes one key and does what it means on a menu.
//!
//! Nothing happens (`Pressed::Nothing`) while an item is being edited (a service box turned
//! over) or the controller page is learning. Otherwise the menu sound and
//! `loop_flags.leave_screen` are cleared and the key is taken (`input::take_key`), first through
//! `menu::handle_yes_no`:
//! - none or 0x38: `Pressed::Nothing`;
//! - Escape (1): `loop_flags.leave_screen`, `draw_work.escape_pressed` and
//!   `captions.left_by_key` set; `Pressed::Escape`;
//! - Enter (0x1C): the Enter menu sound, the last-key time set to the clock, the
//!   enter-skips-name-entry flag set when bits 8 .. 15 of the caller's value are 4, the Enter
//!   click played; `Pressed::Enter`;
//! - any other: the last-key time set, then the four `move_highlight_*` (0x66, 0x6B, 0x69,
//!   0x68), a screenshot (0x52), `loop_flags.toggle` flipped (0x20), 60 added to the service
//!   time budget when bit 14 of the cheat bits is set (0x14); `Pressed::Nothing`.
//!
//! 89 callers: every screen loop. Refuses, on a yes/no screen, a missing keyboard layout
//! (before the key is taken).

use crate::errors::{BadInput, Refused};
use crate::input::key_to_char::has_layout;
use crate::input::take_key::take_key;
use crate::menu::handle_yes_no::handle_yes_no;
use crate::menu::navigation::{Direction, move_highlight};
use crate::owners::Owners;
use rc2k_core::menu::keys::Pressed;

/// `eax` is the caller's value (the screen loops pass the elapsed time); bits 8 .. 15 are read
/// on Enter.
pub fn handle_keys(o: Owners<'static>, eax: u32) -> Result<Pressed, Refused> {
    use rc2k_core::menu::keys::{self, Action, Move};
    let menus = o.menus;
    let nothing = Pressed::Nothing;
    if keys::keys_off(menus.screens.state.service_ui.box_shown.is_set(), || {
        o.screens.controls.learning.get()
    }) {
        return Ok(nothing);
    }
    if menus.screens.flow.asking.is_set() && !has_layout() {
        return Err(BadInput("no keyboard layout").into());
    }
    menus.screens.entry.set_sound(keys::MenuSound::None);
    menus.screens.loop_flags.leave_screen.set(false);
    let key = take_key(o.input);
    if key == 0 {
        return Ok(nothing);
    }
    let eax = eax & !0xFF | key as u32;
    handle_yes_no(
        crate::strings::append_string_by_id::StringOwners::of(o),
        o.input,
        menus,
        eax,
    )?;
    let action = keys::action(key, || menus.play.cheats.cheats.get());
    match action {
        Action::Ignored => return Ok(nothing),
        Action::Escape => {
            menus.screens.loop_flags.leave_screen.set(true);
            menus.screens.draw_work.escape_pressed.set(true);
            menus.tables.captions.left_by_key.set(true);
            return Ok(Pressed::Escape);
        }
        _ => {}
    }
    let clock: u32 = o.system.clocks().frame.get();
    if action == Action::Enter {
        menus.screens.entry.set_sound(keys::MenuSound::Enter);
        menus.play.champ_view.last_key_time.set(clock);
        menus
            .screens
            .flow
            .enter_skips_name_entry
            .set(keys::enter_skips_name_entry(eax));
        crate::sound::play_menu_sound::play_menu_sound_in_game(
            o.sound,
            o.audio,
            o.menus,
            keys::ENTER_CLICK,
        );
        return Ok(Pressed::Enter);
    }
    menus.play.champ_view.last_key_time.set(clock);
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
            let toggle = &menus.screens.loop_flags.toggle;
            toggle.set(!toggle.is_set());
            return Ok(nothing);
        }
        Action::AddTime => {
            menus.screens.event.service_minutes.set(
                menus
                    .screens
                    .event
                    .service_minutes
                    .get()
                    .wrapping_add(keys::ADDED_TIME),
            );
            return Ok(nothing);
        }
        _ => return Ok(nothing),
    };
    move_highlight(o, direction)?;
    Ok(nothing)
}
