//! The controller page's axis learning: while `controls.learning` is 1 (for at most 5 s from
//! the learning start time), the control being learnt takes the axis or the direction the player
//! moves, into the player's profile for the controller.

use crate::errors::{BadInput, Refused};
use crate::input::owner::Input;
use crate::menu::owner::Menus;

/// With 5 s gone the learning stops (`ControlsScreen.learned` -1) and the control stays as it
/// was. The chosen item's prompt (string 0x3BE) is drawn while `DrawWork.blink` is 1. The
/// first button, hat direction or (for accelerate, brake, left and right) axis that moves
/// from the controller's rest goes into the player's profile. Space stops it with nothing, and
/// so does a pad's Back held for a second (let go sooner it is learnt as a button).
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
        // The prompt names what the row takes.
        use rc2k_core::input::profile::Row;
        let prompt = match controls_row(controls) {
            Some(Row::SteeringAxis) => rc2k_core::strings::own::id::PROMPT_STEERING,
            Some(Row::AccelerateAxis | Row::BrakeAxis) => rc2k_core::strings::own::id::PROMPT_AXIS,
            _ => rc2k_core::strings::own::id::PROMPT_BUTTON,
        };
        crate::text::draw_string_by_id::draw_string_by_id_at(
            crate::strings::append_string_by_id::StringOwners::of(o),
            prompt,
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
    let Some((kind, axes)) = crate::input::profiles::controller_kind(input, controller) else {
        return Ok(());
    };
    let current = screens.event.current_player.get();
    let mut now = now;
    let mut found = None;
    if kind == rc2k_core::input::profile::Kind::Pad {
        // A pad's Back held for a second cancels, as Space does; let go sooner it is a button
        // like another to learn. While it is down nothing else is looked at.
        let ticks = crate::system::tick_count::tick_count(&o.system.session.picture_clock);
        match back_hold(input, now.buttons >> BACK_BUTTON & 1 != 0, ticks) {
            BackHold::Held => return Ok(()),
            BackHold::Cancelled => {
                controls.learning.set(0);
                return Ok(());
            }
            BackHold::Tapped if row.takes_axis() => {}
            BackHold::Tapped => {
                found = Some(rc2k_core::input::profile::Learned::Digital(
                    rc2k_core::input::profile::Digital::Button {
                        button: BACK_BUTTON,
                    },
                ));
            }
            BackHold::Up => {}
        }
        now.buttons &= !(1 << BACK_BUTTON);
    }
    let rest = crate::input::profiles::learning_rest(input, &now);
    let found =
        found.or_else(|| rc2k_core::input::profile::detect(&now, &rest, axes, row.takes_axis()));
    if let Some(found) = found {
        crate::input::profiles::with_profile(input, current, controller, |profile| {
            rc2k_core::input::profile::learn(profile, row, found);
        });
        controls.learning.set(0);
    }
    Ok(())
}

/// The row being learnt.
fn controls_row(
    controls: &crate::screens::controls_store::ControlsScreen,
) -> Option<rc2k_core::input::profile::Row> {
    rc2k_core::input::profile::Row::of_control(controls.learned.get())
}

/// The pad's Back in the profiles' numbering of a gamepad's buttons.
const BACK_BUTTON: u8 = 6;

/// How long the pad's Back is held to cancel a learning (ms).
const BACK_CANCEL_MS: u32 = 1000;

/// What the pad's Back is doing in a learning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BackHold {
    Up,
    Held,
    Cancelled,
    Tapped,
}

/// Follows the pad's Back through a learning: `down` now, at `ticks` (ms). Held for
/// `BACK_CANCEL_MS` it cancels; let go before that it was a tap.
fn back_hold(input: &Input, down: bool, ticks: u32) -> BackHold {
    use std::sync::atomic::Ordering::Relaxed;
    let since = &input.learn_back_since;
    let began = since.load(Relaxed);
    match (down, began) {
        (false, 0) => BackHold::Up,
        (false, _) => {
            since.store(0, Relaxed);
            BackHold::Tapped
        }
        (true, 0) => {
            since.store(u64::from(ticks) + 1, Relaxed);
            BackHold::Held
        }
        (true, began) => {
            if ticks.wrapping_sub((began - 1) as u32) >= BACK_CANCEL_MS {
                since.store(0, Relaxed);
                BackHold::Cancelled
            } else {
                BackHold::Held
            }
        }
    }
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

/// The player's controller (none: items 3 to 6 off and string 0x56D), else items 3 to 6 on; the
/// device's name; the shifter switch; item 4 only for a device that has it, with its strength (a
/// percent, or string 0x56D). Refuses no current player record, or no input device for a player
/// that has one.
pub fn draw_controller_page(
    o: crate::owners::Owners<'static>,
    input: &Input,
    canvas: &mut dyn crate::text::draw_text::Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    // The logic is in `rc2k_core::screens::controller_page::controller_page`.
    use rc2k_core::screens::controller_page::{Controller, Font, Work, controller_page};
    let player = controller_page_refuses(menus, input)?;
    let p = player.get();
    let device = p.controller;
    use crate::screens::game_screen::HeldText;
    let mut screen =
        crate::screens::game_screen::GameScreen::<Font, Work, HeldText>::new(canvas, o);
    if device == 0 {
        return controller_page(&mut screen, None);
    }
    // Device n is the controllers' record n - 1.
    let Some(entry) = (device as usize)
        .checked_sub(1)
        .and_then(|n| input.devices.controllers.get(n))
    else {
        return Err(BadInput("a player's device past the controllers").into());
    };
    let c = Controller {
        name: crate::screens::game_screen::held_field(&entry.name)?,
        has_strength: entry.force_feedback.get() == 1,
        mode: crate::input::profiles::with_profile(
            input,
            menus.screens.event.current_player.get(),
            device,
            |profile| u32::from(profile.shifter),
        )
        .unwrap_or(0),
        strength: p.strength,
        icons: rc2k_core::strings::own::id::ICON_STYLES
            + crate::input::profiles::with_profile(
                input,
                menus.screens.event.current_player.get(),
                device,
                |profile| profile.button_icons.place() as u32,
            )
            .unwrap_or(0),
    };
    controller_page(&mut screen, Some(&c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pad_back_held_a_second_cancels_and_a_tap_is_learnt() {
        let input = Input::new();
        assert_eq!(back_hold(&input, false, 100), BackHold::Up);
        assert_eq!(back_hold(&input, true, 200), BackHold::Held);
        assert_eq!(back_hold(&input, true, 1199), BackHold::Held);
        assert_eq!(back_hold(&input, true, 1200), BackHold::Cancelled);
        // let go after the cancel: nothing more
        assert_eq!(back_hold(&input, false, 1300), BackHold::Up);
        // a short press and release
        assert_eq!(back_hold(&input, true, 2000), BackHold::Held);
        assert_eq!(back_hold(&input, false, 2300), BackHold::Tapped);
        assert_eq!(back_hold(&input, false, 2400), BackHold::Up);
    }
}
