//! The controller page's axis learning (`screens::learn_axis` -> 0x46B4AE,
//! `screens::classify_controller`): while `controls.learning` is 1 (for
//! at most 3 s from
//! the learning start time), the player's control the control being learnt (1 .. ) learns the axis or the
//! direction the player moves, into the player's +0x50 table.

use crate::errors::{BadInput, Refused};
use crate::input::owner::Input;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::screens::controls_store::ControlsScreen;
use crate::screens::owner::screens as screens_owner;

/// `screens::classify_controller` (0x46B27A): from the controller's four values (ecx x, edx
/// y, eax z, ebx the buttons): the button (its number + 1, when exactly one is down), and
/// each of left (x below 0), right (x above 0), y and z shifted into its history
/// (`ControlsScreen.history[0]` ..: a 1 for "under 0x80"); a direction held over 0x1D of the last 32 polls
/// while the others were not (its history empty) gives 0x400 / 0x800 / 0x100 / 0x200 with the
/// button, else 0; into `DrawWork.figure`.
pub fn classify_controller(
    controls: &ControlsScreen,
    screens: &Screens,
    z: u32,
    buttons: u32,
    x: u32,
    y: u32,
) {
    // The logic is in `rc2k_core::input::devices::classify`.
    let (history, found) = rc2k_core::input::devices::classify(
        buttons,
        x,
        y,
        z,
        std::array::from_fn(|n| controls.history[n].get()),
    );
    for (cell, v) in controls.history.iter().zip(history) {
        cell.set(v);
    }
    screens.draw_work.figure.set(found);
}

/// `screens::learn_axis` (0x46B131): see the header. With 3 s gone the learning stops
/// (`ControlsScreen.learned` -1). The chosen item's prompt (string 0x3BE at x 0x1B8, the item's y)
/// while `DrawWork.blink` is 1. A player of kind 3 (+0x34) learning control 1 or 2: the axis
/// moved (`input::axis_moved` 1) into +0x3B or +0x3A, `Devices.records[0].axis_code` its +0x38, the found
/// 0x200 or 0x100; else the controller read and classified. Space (key 0x39) stops it with
/// nothing; anything found goes to the player's +0x50 + 4 x (control - 1). The original keeps
/// every register. Refuses (before anything) a player without its record.
pub fn learn_axis(menus: &Menus, input: &Input) -> Result<(), Refused> {
    let controls = &screens_owner().controls;
    let screens = &menus.screens;
    if controls.learning.get() != 1 {
        return Ok(());
    }
    let Some(player) = crate::menu::player::player(menus, screens.event.current_player.get())
    else {
        return Err(crate::errors::BadInput("no current player record").into());
    };
    screens.draw_work.figure.set(0);
    // The logic is in `rc2k_core::screens::learning` (`timed_out`, `by_axis`, `pedal`,
    // `CANCEL_KEY`, `binding`).
    use rc2k_core::screens::learning as l;
    let since = crate::system::tick_count::tick_count().wrapping_sub(controls.learn_start.get());
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
            .item(screens.loop_flags.item.get())?;
        crate::text::draw_string_by_id::draw_string_by_id_at(
            0x3BE,
            crate::gfx::typefaces::fonts::LIST,
            0x1B8,
            item.fixed().alignment_or_text_y,
            &mut crate::text::draw_text::GameCanvas,
        )?;
    }
    let mut take = true;
    let control = controls.learned.get();
    if l::by_axis(player.get().sensitivity_level, control) {
        let m = crate::input::axes::axis_moved(1, input)?;
        if !m.carry {
            let (byte, found) = l::pedal(control);
            player.update(|p| p.axis_codes[byte] = m.eax as u8);
            input.devices.records[0]
                .axis_code
                .set(u32::from_le_bytes(player.get().axis_codes));
            screens.draw_work.figure.set(found);
            controls.learning.set(0);
            take = false;
        }
    } else {
        let [ecx, edx, eax, ebx] = crate::input::axes::first_device_state(input)?;
        classify_controller(controls, screens, eax, ebx, ecx, edx);
    }
    if take {
        if crate::input::take_key::take_key() == l::CANCEL_KEY {
            screens.draw_work.figure.set(0);
            controls.learning.set(0);
        } else if screens.draw_work.figure.get() == 0 {
            return Ok(());
        }
    }
    let Some(slot) = l::binding(controls.learned.get()) else {
        return Err(crate::errors::BadInput("a learned control past the bindings").into());
    };
    let figure = screens.draw_work.figure.get();
    player.update(|p| p.set_binding(slot, figure));
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

/// `screens::draw_controller_page` (0x46A454): the player's controller (+0x2C, 0 none: items
/// 3 .. 8 off and string 0x56D): items 3 .. 8 on; the device's name (record +0x30, the record
/// in `DrawWork.scratch_0c`); the mode (string 0x56D + player +0x3C); item 4 only for a device that
/// has it (+0x1C 1) with its strength (player +0x48 as a percent, or string 0x56D); the
/// sensitivity (string 0x3A1 + player +0x34); two sliders (player +0x40 and +0x44: their
/// fills clipped in, their ends at `DrawWork.slider_a_start` .. `DrawWork.slider_b_end`); and the stick's place
/// now (x of the first device's state, x 100 / 256 from 0x21A; a pressed button sprite 9,
/// else 8; `hhmm` the buttons). The sprites' palettes are ebx as the original leaves it.
/// Refuses (before anything) no current player record, or no input
/// device for a player that has one; an x whose x 100 is 0 is the original's fatal error
/// 0x80701010.
pub fn draw_controller_page(
    menus: &Menus,
    input: &Input,
    canvas: &mut dyn crate::text::draw_text::Canvas,
) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::controller_page::controller_page`.
    use rc2k_core::screens::controller_page::{Controller, Font, Work, controller_page};
    {
        let player = controller_page_refuses(menus, input)?;
        let p = player.get();
        let device = p.controller;
        use crate::screens::game_screen::HeldText;
        let mut screen =
            crate::screens::game_screen::GameScreen::<Font, Work, HeldText>::new(canvas);
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
            // Kept in `draw_work.scratch_0c`: the record's number among the controllers.
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
