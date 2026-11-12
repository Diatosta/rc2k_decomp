//! `menu::handle_keys` (0x465FCC): takes one key and does what it means on a menu.
//!
//! Nothing happens (eax 1) while an item is being edited (a service box turned over) or
//! the controller page is learning (`controls.learning` 1). Otherwise `option_work.menu_sound`
//! = 0x6F, `loop_flags.leave_screen` = 0, and the key
//! taken (`input::take_key`, which changes only al), first through `menu::handle_yes_no`:
//! - none: eax 1;
//! - 0x38: nothing;
//! - 1 (Escape): `loop_flags.leave_screen`, `draw_work.escape_pressed` and `captions.left_by_key` set to 1; eax 2;
//! - 0x1C (Enter): `option_work.menu_sound` 0x14D, the last-key time (`champ_view.last_key_time`) the clock,
//!   `g_enterSkipsNameEntry` 1 if ah (what the caller had in eax) is 4, else 0, the sound
//!   2; eax 3;
//! - any other: the last-key time the clock, then 0x66, 0x6B, 0x69, 0x68 the four
//!   `move_highlight_*`, 0x52 a screenshot (`gfx::take_screenshot`), 0x20 flips `loop_flags.toggle`,
//!   0x14 adds 60 to the service time budget (`event.service_minutes`) if bit 14 of the cheat bits is
//!   set; eax 1.
//!
//! The code after the Escape branch that could return 0 is never reached.
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

// ---- game data ----

/// `eax` is the caller's eax, whose bits 8 .. 15 are read on Enter (the screen loops: the
/// elapsed time).
pub fn handle_keys(o: Owners<'static>, eax: u32) -> Result<Pressed, Refused> {
    // The logic is in `rc2k_core::menu::keys` (`keys_off`, `action`, `enter_skips_name_entry`).
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
    menus.screens.option_work.set_sound(keys::MenuSound::None);
    menus.screens.loop_flags.leave_screen.set(false);
    let key = take_key();
    if key == 0 {
        return Ok(nothing);
    }
    let eax = eax & !0xFF | key as u32;
    handle_yes_no(menus, eax)?;
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
    let clock: u32 = o.system.session.clocks.frame.get();
    if action == Action::Enter {
        menus.screens.option_work.set_sound(keys::MenuSound::Enter);
        menus.play.champ_view.last_key_time.set(clock);
        menus
            .screens
            .flow
            .enter_skips_name_entry
            .set(keys::enter_skips_name_entry(eax));
        crate::sound::play_menu_sound::play_menu_sound_in_game(o, keys::ENTER_CLICK);
        return Ok(Pressed::Enter);
    }
    menus.play.champ_view.last_key_time.set(clock);
    let direction = match action {
        Action::Move(Move::Up) => Direction::Up,
        Action::Move(Move::Down) => Direction::Down,
        Action::Move(Move::Right) => Direction::Right,
        Action::Move(Move::Left) => Direction::Left,
        Action::Screenshot => {
            crate::gfx::take_screenshot::take_screenshot(o.gfx)?;
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
