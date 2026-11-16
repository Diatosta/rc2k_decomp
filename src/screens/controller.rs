//! The controller page's axis learning: while `controls.learning` is 1 (for at most 3 s from
//! the learning start time), the control being learnt takes the axis or the direction the player
//! moves, into the player's binding table.

use crate::errors::{BadInput, Refused};
use crate::input::owner::Input;
use crate::menu::owner::Menus;
use crate::screens::controls_store::ControlsScreen;

/// From the controller's four values (x, y, z and the buttons): the button (its number + 1,
/// when exactly one is down), and each of left (x below 0), right (x above 0), y and z shifted
/// into its history (`ControlsScreen.history[0]` ..: a 1 for "under 0x80"); a direction held
/// over 0x1D of the last 32 polls while the others were not (its history empty) gives 0x400 /
/// 0x800 / 0x100 / 0x200 with the button, else 0, returned.
pub fn classify_controller(controls: &ControlsScreen, z: u32, buttons: u32, x: u32, y: u32) -> u32 {
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
    found
}

/// With 3 s gone the learning stops (`ControlsScreen.learned` -1). The chosen item's prompt
/// (string 0x3BE) is drawn while `DrawWork.blink` is 1. A player of kind 3 learning control 1
/// or 2: the axis moved (`input::axis_moved` 1) goes into the player's axis fields, the found
/// code 0x200 or 0x100; else the controller is read and classified. Space stops it with
/// nothing; anything found goes to the player's binding table. Refuses a player without its
/// record.
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
    // The code the learning found, 0 for none.
    let mut figure = 0;
    // The logic is in `rc2k_core::screens::learning` (`timed_out`, `by_axis`, `pedal`,
    // `CANCEL_KEY`, `binding`).
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
            figure = found;
            controls.learning.set(0);
            take = false;
        }
    } else {
        let [ecx, edx, eax, ebx] = crate::input::axes::first_device_state(input)?;
        figure = classify_controller(controls, eax, ebx, ecx, edx);
    }
    if take {
        if crate::input::take_key::take_key(input) == l::CANCEL_KEY {
            figure = 0;
            controls.learning.set(0);
        } else if figure == 0 {
            return Ok(());
        }
    }
    let Some(slot) = l::binding(controls.learned.get()) else {
        return Err(crate::errors::BadInput("a learned control past the bindings").into());
    };
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
