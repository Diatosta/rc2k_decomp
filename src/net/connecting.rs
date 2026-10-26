//! `net::connecting` (0x417688): an empty frame (`screens::run_frame`, eax 0), then on the
//! locked surface the background and "connecting" (string 0x14F, `stru_51E9A4` at 0x316,
//! 0x6F), shown, a second's sleep (none when the lock fails); then connection eax made
//! (`net::init_connection`).
//!
//! In:  eax = the connection's number
//! Out: every register kept (`pusha` .. `popa`); the flags `net::init_connection` leaves
//!      (carry: it failed)
//! Caller: `net::start_link`. Its calls through `part` (the frames through the game).
//! Shadow mode runs the original alone. Self-test: a copy whose calls go to stand-ins,
//! against the Rust version, the lock and the connection failing or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{CARRY_FLAG, RawHook, Regs, SelfTest};
use crate::net::lobby::{
    COPY_PICTURE, DRAW_STRING, LOCK, Part, RUN_FRAME, SHOW_FRAME, SLEEP, UNLOCK, answer,
    lobby_entry, run_both,
};

pub const STATUS_FONT: usize = 0x51E9A4; // stru_51E9A4
const INIT_CONNECTION: u32 = 0x418794; // net::init_connection

/// An empty frame, then string `id` shown on the background and a second's sleep (not
/// when the lock fails). Shared with `net::hosting`, `net::joining`, `net::stopping`.
pub fn status(r: &mut Regs, id: u32, part: Part) -> Result<(), Refused> {
    r.eax = 0;
    part(RUN_FRAME, r)?;
    part(LOCK, r)?;
    if r.carry() {
        return Ok(());
    }
    crate::net::message_frame::string_frame_locked(r, id, STATUS_FONT, 0x316, 0x6F, part)?;
    r.eax = 1000;
    part(SLEEP, r)
}

/// The Rust version: the registers it leaves (the entry's, the call's flags).
pub fn connecting(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.esi = r.eax;
    status(&mut r, 0x14F, part)?;
    r.eax = r.esi;
    part(INIT_CONNECTION, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, connecting)
}

/// The test of the three status screens: a copy of `address` (`len` bytes, its first
/// bytes `entry`, its calls at `calls` to `places`, the last the call after), the lock and
/// the last call failing or not.
pub fn status_test(
    hook: &RawHook,
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    last: u32,
    rust: fn(&Regs, Part) -> Result<Regs, Refused>,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let places = [
        RUN_FRAME,
        LOCK,
        COPY_PICTURE,
        DRAW_STRING,
        UNLOCK,
        SHOW_FRAME,
        SLEEP,
        last,
    ];
    let copy = match crate::net::lobby::copy(address, len, entry, calls, &places) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    for (lock_fails, last_fails) in [(false, false), (true, false), (false, true)] {
        run_both(
            &mut t,
            &format!("the lock failing {lock_fails}, the call {last_fails}"),
            copy.addr(),
            Regs {
                eax: 3,
                ..Regs::SENTINEL
            },
            &[],
            CARRY_FLAG,
            &[],
            &|| {
                answer(Box::new(move |place, r| {
                    r.set_carry((place == LOCK && lock_fails) || (place == last && last_fails))
                }))
            },
            &rust,
        );
    }
    t.finish()
}
