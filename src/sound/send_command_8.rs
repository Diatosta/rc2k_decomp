/// The master volume.
pub fn send_command_8(audio: &crate::audio::owner::Audio, volume: u32) {
    crate::sound::senders::send(audio, rc2k_core::audio::commands::Command::Volume(volume));
}
