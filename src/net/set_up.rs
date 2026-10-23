//! `net::set_up` (0x41848E): what the network game needs before anything else: the
//! receiving thread's event and the mutex, and whether a lobby launched the game.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! With `g_noticeKind` (a byte) 2, `dword_52ABD0` (signed) held to 4 at most. An event (auto reset,
//! not signalled: `threading::create_event` with eax 0, ebx 0) to `dword_52AB68`, none:
//! fatal error 0x80168501; a mutex (`threading::create_mutex`) to `dword_52AB6C`, none:
//! fatal error 0x80168502. Then `sub_418E23` (through `part`): DirectPlay's lobby asked
//! (`CoCreateInstance`) whether the game was launched by one ("NET: Lobby launch
//! detected", `dword_5AB290` 1); the platform's (DirectPlay), the game's in the game,
//! nothing in the standalone program (no lobby: `dword_5AB290` stays 0).
//! One caller, `start` (0x4010A2).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::{FatalError, Refused};
use crate::framework::{Regs, game_addr, rva};
use crate::threading::create_event::{create_event, handle_flags};
use crate::threading::create_mutex::create_mutex;

// ---- game data ----

const NOTICE_KIND_RVA: usize = rva(0x4E07A0); // g_noticeKind
const PLAYERS_RVA: usize = rva(0x52ABD0); // dword_52ABD0: held to 4 with notice kind 2
pub const EVENT_RVA: usize = rva(0x52AB68); // dword_52AB68: the receiving thread's event
const MUTEX_RVA: usize = rva(0x52AB6C); // dword_52AB6C
const NO_EVENT: FatalError = FatalError {
    code: 0x8016_8501,
    message: "",
};
const NO_MUTEX: FatalError = FatalError {
    code: 0x8016_8502,
    message: "",
};

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// Where the variables are (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct NetVars {
    /// g_noticeKind, a byte
    pub notice_kind: u32,
    pub players: u32,
    pub event: u32,
    pub mutex: u32,
}

impl NetVars {
    pub fn game() -> Self {
        let a = |r| game_addr(r) as u32;
        Self {
            notice_kind: a(NOTICE_KIND_RVA),
            players: a(PLAYERS_RVA),
            event: a(EVENT_RVA),
            mutex: a(MUTEX_RVA),
        }
    }
}

// ---- Rust version ----

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn set(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

pub unsafe fn set_up(v: &NetVars, part: Part) -> Result<(), Refused> {
    unsafe {
        if (v.notice_kind as *const u8).read() == 2 && get(v.players) as i32 > 4 {
            set(v.players, 4);
        }
        let mut r = Regs::default();
        part(0, &mut r)?;
        if r.carry() {
            return Err(NO_EVENT.into());
        }
        set(v.event, r.eax);
        part(1, &mut r)?;
        if r.carry() {
            return Err(NO_MUTEX.into());
        }
        set(v.mutex, r.eax);
        part(2, &mut r)?;
        Ok(())
    }
}

/// In the game: the handles from the Rust versions, the lobby through the game; in the
/// standalone program there is no lobby.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        0 | 1 => {
            r.eax = if place == 0 {
                create_event(0, 0)
            } else {
                create_mutex()
            };
            r.set_carry(handle_flags(r.eax).0);
        }
        _ => {}
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { set_up(&NetVars::game(), &mut game_part) }?;
    Ok(())
}
