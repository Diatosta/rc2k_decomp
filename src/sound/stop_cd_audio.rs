//! `sound::stop_cd_audio`: `CdAudio.track` (the track playing) set to 0 and the CD audio stopped
//! five times through `sound::stop_cd_track`.
//!
//! Six callers: the CD audio's functions and `system::main_loop`.

// ---- game data ----

const STOPS: u32 = 5;

pub fn stop_cd_audio(sound: &crate::sound::owner::Sound) {
    sound.menu.cd_audio.get().track.set(0);
    for _ in 0..STOPS {
        crate::sound::stop_cd_track::stop_cd_track_in_game(sound);
    }
}
