//! `sound::stop_cd_track` (0x40FCC8): with the CD audio open (`dword_518108` 1),
//! `mciSendCommandA(mciId, MCI_STOP, 0, 0)`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! Called by `sound::stop_cd_audio` (three calls and two more by its `call $+5` into this).
//! The MCI call is the platform's: in the game through the import's thunk (0x4D5E7E); in
//! the standalone program nothing (no CD audio; in this build `dword_518108` never becomes
//! 1: `sound::open_cd_audio`). Shadow mode runs the original alone (the device is stopped
//! once). Self-test: a copy whose two variables are the test's and whose MCI call goes to a
//! stand-in that notes its arguments, against the Rust version, open and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_addr, rva};

// ---- game data ----

const CD_OPEN_RVA: usize = rva(0x518108); // dword_518108: the CD audio is open
const MCI_ID_RVA: usize = rva(0x518118); // mciId
const MCI_STOP: u32 = 0x808;

/// `mciSendCommandA(device, message, parameter 1, parameter 2)`.
pub type SendCommand<'a> = &'a mut dyn FnMut(u32, u32, u32, u32) -> u32;

#[derive(Clone, Copy)]
pub struct CdVars {
    pub open: u32,
    pub device: u32,
}

impl CdVars {
    pub fn game() -> Self {
        Self {
            open: game_addr(CD_OPEN_RVA) as u32,
            device: game_addr(MCI_ID_RVA) as u32,
        }
    }
}

// ---- Rust version ----

pub unsafe fn stop_cd_track(v: &CdVars, send: SendCommand) {
    unsafe {
        if (v.open as *const u32).read_unaligned() == 1 {
            send((v.device as *const u32).read_unaligned(), MCI_STOP, 0, 0);
        }
    }
}

/// `mciSendCommandA`: the standalone plays no CD, so nothing.
pub fn game_send(_device: u32, _message: u32, _a: u32, _b: u32) -> u32 {
    0
}

/// In the game: its variables and MCI.
pub fn stop_cd_track_in_game() {
    unsafe { stop_cd_track(&CdVars::game(), &mut game_send) };
}
