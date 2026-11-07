//! `sound::stop_cd_audio` (0x40FCAA): `dword_518110` (the track playing) 0 and the CD
//! audio stopped five times: three calls of `sound::stop_cd_track`, then `call $+5` into
//! it, whose return lands on it once more, and from there back to the caller.
//!
//! In:  nothing
//! Out: every register preserved (`stop_cd_track` keeps them all)
//! Six callers: the CD audio's functions (0x40F8BB .. 0x40FDB9) and `system::main_loop`
//! (0x415439).

// ---- game data ----

const STOPS: u32 = 5;

// ---- Rust version ----

pub fn stop_cd_audio(sound: &crate::sound::owner::Sound) {
    sound.menu.cd_audio.track.set(0);
    for _ in 0..STOPS {
        crate::sound::stop_cd_track::stop_cd_track_in_game(sound);
    }
}
