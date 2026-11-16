//! The controller page's axis learning: while `controls.learning` is 1 (for at most 5 s from
//! the learning start time), the control being learnt takes the axis or the direction the player
//! moves, into the player's profile for the controller.

use crate::errors::{BadInput, Refused};
use crate::input::owner::Input;
use crate::menu::owner::Menus;

/// With 5 s gone the learning stops (`ControlsScreen.learned` -1) and the control stays as it
/// was. The chosen item's prompt (string 0x3BE) is drawn while `DrawWork.blink` is 1. The
/// first button, hat direction or (for accelerate, brake, left and right) axis that moves
/// from the controller's rest goes into the player's profile. Space stops it with nothing.
/// Refuses a player without its record.
pub fn learn_axis(
    o: crate::owners::Owners<'static>,
    input: &Input,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    let menus = o.menus;
    let controls = &menus.screens.controls;
    let screens = &menus.screens;
    if controls.learning.get() != 1 {
        return Ok(());
    }
    let Some(player) = crate::menu::player::player(menus, screens.event.current_player.get())
    else {
        return Err(crate::errors::BadInput("no current player record").into());
    };
    // The logic is in `rc2k_core::screens::learning` (`timed_out`, `CANCEL_KEY`).
    use rc2k_core::screens::learning as l;
    let since = crate::system::tick_count::tick_count(&o.system.session.picture_clock)
        .wrapping_sub(controls.learn_start.get());
    if l::timed_out(since) {
        controls.learned.set(l::TIMED_OUT);
        controls.learning.set(0);
        return Ok(());
    }
    if screens.draw_work.blink.is_set() {
        let item = screens
            .loop_flags
            .menu
            .get()?
            .item(&menus.tables.menus, screens.loop_flags.item.get())?;
        crate::text::draw_string_by_id::draw_string_by_id_at(
            crate::strings::append_string_by_id::StringOwners::of(o),
            0x3BE,
            crate::gfx::typefaces::fonts::LIST,
            0x1B8,
            item.fixed().alignment_or_text_y,
            &mut crate::text::draw_text::GameCanvas::new(gfx),
        )?;
    }
    // Space cancels; else what moved from the controller's rest is learnt into the player's
    // profile for the controller.
    if crate::input::take_key::take_key(input) == l::CANCEL_KEY {
        controls.learning.set(0);
        return Ok(());
    }
    let control = controls.learned.get();
    let Some(row) = rc2k_core::input::profile::Row::of_control(control) else {
        return Err(crate::errors::BadInput("a learned control past the bindings").into());
    };
    let controller = player.get().controller;
    let Some(now) = crate::input::profiles::first_reading(input) else {
        return Ok(());
    };
    let rest = crate::input::profiles::learning_rest(input, &now);
    let Some((_, axes)) = crate::input::profiles::controller_kind(input, controller) else {
        return Ok(());
    };
    if let Some(found) = rc2k_core::input::profile::detect(&now, &rest, axes, row.takes_axis()) {
        let current = screens.event.current_player.get();
        crate::input::profiles::with_profile(input, current, controller, |profile| {
            rc2k_core::input::profile::learn(profile, row, found);
        });
        controls.learning.set(0);
    }
    Ok(())
}

/// What the controller page reads: refused before anything.
pub fn controller_page_refuses<'m>(
    menus: &'m Menus,
    input: &Input,
) -> Result<crate::menu::standings::PlayerCell<'m>, Refused> {
    let Some(player) = crate::menu::player::player(menus, menus.screens.event.current_player.get())
    else {
        return Err(crate::errors::BadInput("no current player record").into());
    };
    let device = player.get().controller;
    if device != 0 && input.devices.device(0).is_none() {
        return Err(crate::errors::BadInput("no input device").into());
    }
    Ok(player)
}

/// The player's controller (none: items 3 to 8 off and string 0x56D), else items 3 to 8 on;
/// the device's name; the mode; item 4 only for a device that has it, with its strength (a
/// percent, or string 0x56D); the sensitivity; two sliders (their fills clipped in); and the
/// stick's place now (a pressed button sprite 9, else 8). The sprites' palette is unused.
/// Refuses no current player record, or no input device for a player that has one; an x whose
/// x 100 is 0 is a fatal error.
pub fn draw_controller_page(
    o: crate::owners::Owners<'static>,
    input: &Input,
    canvas: &mut dyn crate::text::draw_text::Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::controller_page::controller_page`.
    use rc2k_core::screens::controller_page::{Controller, Font, Work, controller_page};
    {
        let player = controller_page_refuses(menus, input)?;
        let p = player.get();
        let device = p.controller;
        use crate::screens::game_screen::HeldText;
        let mut screen =
            crate::screens::game_screen::GameScreen::<Font, Work, HeldText>::new(canvas, o);
        if device == 0 {
            return controller_page(&mut screen, None);
        }
        // Device n is the controllers' record n - 1 (0x50 bytes; +0x30 its name, +0x1C 1
        // with a strength).
        let Some(entry) = (device as usize)
            .checked_sub(1)
            .and_then(|n| input.devices.controllers.get(n))
        else {
            return Err(BadInput("a player's device past the controllers").into());
        };
        let [stick_x, _, _, buttons] = crate::input::axes::first_device_state(input)?;
        let c = Controller {
            // The record's number among the controllers.
            record: device - 1,
            name: crate::screens::game_screen::held_field(&entry.name)?,
            has_strength: entry.force_feedback.get() == 1,
            mode: p.controller_mode,
            strength: p.strength,
            sensitivity: p.sensitivity_level,
            first: p.end_zone_pct,
            second: p.centre_dead_zone_pct,
            stick_x,
            buttons,
        };
        controller_page(&mut screen, Some(&c))
    }
}
