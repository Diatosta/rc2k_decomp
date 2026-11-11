/// The master volume.
pub fn send_command_8(volume: u32) {
    crate::sound::senders::send(rc2k_core::audio::commands::Command::Volume(volume));
}
