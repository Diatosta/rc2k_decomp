/// The sample file `name` (its `audio::store::name_key`; the original sent the name's
/// address) played at `volume`.
pub fn start_stream(name: u32, volume: u32) {
    crate::sound::senders::send_and_wait(rc2k_core::audio::commands::Command::Stream {
        path: name,
        volume,
        action: 2,
    });
}
