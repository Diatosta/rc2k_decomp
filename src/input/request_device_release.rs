//! Command 2: the input thread ends the effects and gives both devices back
//! (`input::device_command`). The same store as `race::release_controls`.

/// See the header.
pub fn request_device_release(input: &crate::input::owner::Input) {
    input.devices.input_command.set(2);
}
