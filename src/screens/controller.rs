//! The controller page's axis learning (`sub_46B131` -> `sub_46B4AE`, `sub_46B27A`; stream
//! A, part F1, 2026-10-18): while `dword_7551F4` is 1 (for at most 3 s from
//! `dword_7551F8`), the player's control `dword_7551FC` (1 .. ) learns the axis or the
//! direction the player moves, into the player's +0x50 table.

use crate::errors::Refused;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::screens::controls_store::CONTROLS_SCREEN as C;

/// `screens::classify_controller` (0x46B27A): from the controller's four values (ecx x, edx
/// y, eax z, ebx the buttons): the button (its number + 1, when exactly one is down), and
/// each of left (x below 0), right (x above 0), y and z shifted into its history
/// (`dword_7551E4` ..: a 1 for "under 0x80"); a direction held over 0x1D of the last 32 polls
/// while the others were not (its history empty) gives 0x400 / 0x800 / 0x100 / 0x200 with the
/// button, else 0; into `dword_7559EC`. Every register kept.
pub fn classify_controller(z: u32, buttons: u32, x: u32, y: u32) {
    // Converted: `rc2k_core::input::devices::classify`.
    let (history, found) = rc2k_core::input::devices::classify(
        buttons,
        x,
        y,
        z,
        std::array::from_fn(|n| C.history[n].get()),
    );
    for (cell, v) in C.history.iter().zip(history) {
        cell.set(v);
    }
    DRAW_WORK.figure.set(found);
}

/// `screens::learn_axis` (0x46B131): see the header. With 3 s gone the learning stops
/// (`dword_7551FC` -1). The chosen item's prompt (string 0x3BE at x 0x1B8, the item's y)
/// while `dword_7559A4` is 1. A player of kind 3 (+0x34) learning control 1 or 2: the axis
/// moved (`input::axis_moved` 1) into +0x3B or +0x3A, `dword_4F6F08` its +0x38, the found
/// 0x200 or 0x100; else the controller read and classified. Space (key 0x39) stops it with
/// nothing; anything found goes to the player's +0x50 + 4 x (control - 1). The original keeps
/// every register. Refuses (before anything) a player without its record.
pub unsafe fn learn_axis() -> Result<(), Refused> {
    unsafe {
        if C.learning.get() != 1 {
            return Ok(());
        }
        let player = crate::menu::player::player_at(EVENT_STATE.current_player.get());
        if player == 0 {
            return Err(crate::errors::BadInput("no current player record").into());
        }
        DRAW_WORK.figure.set(0);
        // Converted: `rc2k_core::screens::learning` (`timed_out`, `by_axis`, `pedal`,
        // `CANCEL_KEY`, `binding`).
        use rc2k_core::screens::learning as l;
        let since = crate::system::tick_count::tick_count().wrapping_sub(C.learn_start.get());
        if l::timed_out(since) {
            C.learned.set(l::TIMED_OUT);
            C.learning.set(0);
            return Ok(());
        }
        if DRAW_WORK.blink.get() == 1 {
            let items = LOOP_FLAGS
                .menu
                .get()
                .wrapping_add(4)
                .wrapping_add(LOOP_FLAGS.item.get().wrapping_sub(1).wrapping_mul(0x5C));
            crate::text::draw_string_by_id::draw_string_by_id_at(
                0x3BE,
                crate::gfx::typefaces::fonts::LIST.as_ptr(),
                0x1B8,
                crate::pointers::GamePtr::new(items + 0xC)
                    .read::<u32>("controller: a game value")? as i32,
                &mut crate::text::draw_text::GameCanvas,
            )?;
        }
        let mut take = true;
        let control = C.learned.get();
        if l::by_axis(
            crate::pointers::GamePtr::new(player + 0x34).read::<u32>("controller: a game value")?,
            control,
        ) {
            let m = crate::input::axes::axis_moved(1)?;
            if !m.carry {
                let (byte, found) = l::pedal(control);
                crate::pointers::GamePtr::new(player + byte)
                    .write::<u8>(m.eax as u8, "a controller binding")?;
                crate::input::store::DEVICES.records[0].set_u32(
                    0x18,
                    crate::pointers::GamePtr::new(player + 0x38)
                        .read::<u32>("controller: a game value")?,
                );
                DRAW_WORK.figure.set(found);
                C.learning.set(0);
                take = false;
            }
        } else {
            let [ecx, edx, eax, ebx] = crate::input::axes::first_device_state()?;
            classify_controller(eax, ebx, ecx, edx);
        }
        if take {
            if crate::input::take_key::take_key() == l::CANCEL_KEY {
                DRAW_WORK.figure.set(0);
                C.learning.set(0);
            } else if DRAW_WORK.figure.get() == 0 {
                return Ok(());
            }
        }
        let slot = player.wrapping_add(l::binding(C.learned.get()));
        crate::pointers::GamePtr::new(slot)
            .write::<u32>(DRAW_WORK.figure.get(), "controller: a game value")?;
        Ok(())
    }
}

/// What the controller page reads: refused before anything.
pub fn controller_page_refuses() -> Result<u32, Refused> {
    let player = crate::menu::player::player_at(EVENT_STATE.current_player.get());
    if player == 0 {
        return Err(crate::errors::BadInput("no current player record").into());
    }
    let device =
        crate::pointers::GamePtr::new(player + 0x2C).read::<u32>("controller: a game value")?;
    if device != 0 && crate::input::store::DEVICES.input_devices[0].get() == 0 {
        return Err(crate::errors::BadInput("no input device").into());
    }
    Ok(player)
}

/// `screens::draw_controller_page` (0x46A454): the player's controller (+0x2C, 0 none: items
/// 3 .. 8 off and string 0x56D): items 3 .. 8 on; the device's name (record +0x30, the record
/// in `dword_755A0C`); the mode (string 0x56D + player +0x3C); item 4 only for a device that
/// has it (+0x1C 1) with its strength (player +0x48 as a percent, or string 0x56D); the
/// sensitivity (string 0x3A1 + player +0x34); two sliders (player +0x40 and +0x44: their
/// fills clipped in, their ends at `dword_7559DC` .. `dword_7559E8`); and the stick's place
/// now (x of the first device's state, x 100 / 256 from 0x21A; a pressed button sprite 9,
/// else 8; `hhmm` the buttons). The sprites' palettes are ebx as the original leaves it.
/// Every register kept. Refuses (before anything) no current player record, or no input
/// device for a player that has one; an x whose x 100 is 0 is the original's fatal error
/// 0x80701010.
pub fn draw_controller_page(
    canvas: &mut dyn crate::text::draw_text::Canvas,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::controller_page::controller_page`.
    use rc2k_core::screens::controller_page::{Controller, Font, Work, controller_page};
    {
        let player = controller_page_refuses()?;
        let fault = crate::menu::event_flow::Fault::default();
        let f = |o: u32| fault.get(player + o);
        let device = f(0x2C);
        fault.check()?;
        let mut screen = crate::screens::game_screen::GameScreen::<Font, Work>::new(canvas);
        if device == 0 {
            return controller_page(&mut screen, None);
        }
        let record = crate::input::store::DEVICES.controllers[0].addr()
            + device.wrapping_sub(1).wrapping_mul(0x50);
        let [stick_x, _, _, buttons] = crate::input::axes::first_device_state()?;
        let c = Controller {
            record,
            name: record + 0x30,
            has_strength: fault.get(record + 0x1C) == 1,
            mode: f(0x3C),
            strength: f(0x48),
            sensitivity: f(0x34),
            first: f(0x40),
            second: f(0x44),
            stick_x,
            buttons,
        };
        fault.check()?;
        controller_page(&mut screen, Some(&c))
    }
}
