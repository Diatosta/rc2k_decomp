//! `sound::stop_cd_track` (0x40FCC8): with the CD audio open (`dword_518108` 1),
//! `mciSendCommandA(mciId, MCI_STOP, 0, 0)`.
//!
//! Called by `sound::stop_cd_audio` (three calls and two more by its `call $+5` into this).
//! The MCI call is the platform's: in the standalone program nothing (no CD audio; in this build `dword_518108` never becomes
//! 1: `sound::open_cd_audio`).

// ---- game data ----

const MCI_STOP: u32 = 0x808;

/// `mciSendCommandA(device, message, parameter 1, parameter 2)`.
pub type SendCommand<'a> = &'a mut dyn FnMut(u32, u32, u32, u32) -> u32;

// ---- Rust version ----

pub fn stop_cd_track(send: SendCommand, sound: &crate::sound::owner::Sound) {
    if sound.menu.cd_audio.open.get() == 1 {
        send(sound.menu.cd_audio.mci_id.get(), MCI_STOP, 0, 0);
    }
}

/// `mciSendCommandA`: the standalone plays no CD, so nothing.
pub fn game_send(_device: u32, _message: u32, _a: u32, _b: u32) -> u32 {
    0
}

/// In the game: its variables and MCI.
pub fn stop_cd_track_in_game(sound: &crate::sound::owner::Sound) {
    stop_cd_track(&mut game_send, sound);
}
