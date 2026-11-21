//! Sets `Devices.input_command` to 3 (the "createcontrols" step of `load_stage`, its only caller).
//! The meaning of the 3 is not read anywhere.

//! The race's players go with it, for the input thread to set up a device for each.

pub(super) fn create_controls(input: &crate::input::owner::Input, players: u32) {
    input.devices.race_players.set(players);
    input.devices.input_command.set(3);
}
