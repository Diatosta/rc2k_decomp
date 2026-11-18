//! The players' controllers kept by what they are (`rc2k_core::input::device_id`), not by their
//! number in the controllers' table, which only says where a controller came in the list of
//! those plugged in. The number in a player's record is what the game reads; it is worked out
//! from the kept choice whenever the list changes (at start, after a saved game's records come
//! in, when a controller is plugged in or out), and a choice whose controller is not plugged in
//! is kept, the record saying "none" meanwhile, so the player has it back when it comes.

use crate::input::owner::Input;
use crate::menu::owner::Menus;
use rc2k_core::input::device_id::{DeviceId, Listed, find, id_of};
use std::sync::atomic::Ordering;

/// The controllers' table as names and serial numbers, in its order.
pub(crate) fn listed(input: &Input) -> Vec<Listed> {
    let count = input.devices.controller_count.get() as usize;
    input.devices.controllers[..count.min(input.devices.controllers.len())]
        .iter()
        .map(|entry| {
            let info = crate::input::di_objects::device_info(&entry.instance.read());
            Listed {
                name: info.as_ref().map_or_else(String::new, |i| i.name.clone()),
                serial: info.and_then(|i| i.serial),
            }
        })
        .collect()
}

fn chosen(input: &Input) -> std::sync::MutexGuard<'_, [Option<DeviceId>; 4]> {
    input
        .chosen_devices
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Player `player` chose `controller` (0 none, n the table's n-th): kept by what it is.
pub fn note_choice(input: &Input, player: u32, controller: u32) {
    let id = controller
        .checked_sub(1)
        .and_then(|n| id_of(&listed(input), n as usize));
    crate::logfile::program_log::log(&format!(
        "input: player {} chose {}",
        player + 1,
        id.as_ref().map_or_else(
            || "no controller".to_string(),
            |id| format!(
                "\"{}\" (serial number {}, number {} of that name)",
                id.name,
                id.serial.as_deref().unwrap_or("none"),
                id.nth
            )
        )
    ));
    if let Some(slot) = chosen(input).get_mut(player as usize) {
        *slot = id;
    }
}

/// Each player's record given the number of the controller the player chose, where it is in the
/// table now (none while it is not plugged in). A record whose controller has no kept choice
/// (a saved game's, or the first start) has it kept from the number when that is in the table,
/// and none when it is past the table's end (as the game itself does).
pub fn give_chosen(menus: &Menus, input: &Input) {
    let list = listed(input);
    let mut chosen = chosen(input);
    for (n, slot) in chosen.iter_mut().enumerate() {
        let Some(record) = crate::menu::player::player(menus, n as u32) else {
            continue;
        };
        let now = record.get().controller;
        let wanted = match slot {
            Some(id) => find(&list, id).map_or(0, |at| at as u32 + 1),
            None => {
                let listed = rc2k_core::menu::steps::kept_controller(now, list.len() as u32);
                *slot = listed
                    .checked_sub(1)
                    .and_then(|at| id_of(&list, at as usize));
                listed
            }
        };
        // A phone has no keyboard to drive with: the first player's keyboard is the touch
        // screen's controls, or the controller last used (`platform::touch`). The touch pad
        // is never a choice kept for the player: it is what stands in for none.
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let wanted = if n == 0 {
            let touch_pad = crate::platform::touch::is_touch_pad(&list, wanted);
            if touch_pad {
                *slot = None;
            }
            crate::platform::touch::player_one_controller(&list, if touch_pad { 0 } else { wanted })
        } else {
            wanted
        };
        if wanted != now {
            crate::logfile::program_log::log(&format!(
                "input: player {}'s controller {now} -> {wanted} ({})",
                n + 1,
                slot.as_ref()
                    .map_or_else(|| "none".to_string(), |id| id.name.clone())
            ));
            record.update(|p| p.controller = wanted);
        }
    }
}

/// When the input thread has listed controllers since the last call: the players given theirs.
pub fn after_listing(menus: &Menus, input: &Input) {
    if input.devices_listed.swap(false, Ordering::AcqRel) {
        give_chosen(menus, input);
    }
}

/// The kept choices, for the options file.
pub fn kept(input: &Input) -> [Option<DeviceId>; 4] {
    chosen(input).clone()
}

/// The kept choices from the options file, given the players at the next look at the list.
pub fn keep(input: &Input, ids: [Option<DeviceId>; 4]) {
    *chosen(input) = ids;
    input.devices_listed.store(true, Ordering::Release);
}

/// Player 1's controls changed hands on a phone or tablet (a controller used, a finger down in
/// a race, a controller gone): the menus' record is worked out again, and in a race the car's
/// device is set up for the new controls as the race's start does it (effects ended and begun
/// again, the old device given back), so no pedal stays held.
#[cfg(any(target_os = "android", target_os = "ios"))]
pub fn source_changed(
    menus: &Menus,
    input: &Input,
    race: &crate::race::owner::Race,
    in_race: bool,
) {
    give_chosen(menus, input);
    if !in_race || input.devices.race_players.get() > 1 {
        return;
    }
    let controls = &race.shared.state.car(0).controls;
    let now = controls.controller.get();
    let wanted = crate::platform::touch::player_one_controller(&listed(input), now);
    if wanted == now {
        return;
    }
    crate::logfile::program_log::log(&format!(
        "input: the race's player 1 switches from controller {now} to {wanted}"
    ));
    controls.controller.set(wanted);
    // Only when no other command waits for the input thread.
    let _ = input.devices.input_command.atomic().compare_exchange(
        0,
        rc2k_core::input::devices::SWITCH_COMMAND,
        Ordering::SeqCst,
        Ordering::SeqCst,
    );
}
