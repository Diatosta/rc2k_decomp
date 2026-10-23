//! `net::show_message` (0x41745C): message eax shown (`net::message_frame` through
//! `screens::run_frame`), a quarter of a second's sleep, the pending key dropped
//! (`input::take_key`), then frames until a key is pressed.
//!
//! In:  eax = the message's string id (0x154: the connection failed, 0x155: the session)
//! Out: every register kept (`pusha` .. `popa`); the flags of the last `take_key` (no
//!      caller reads them)
//! Caller: `net::start_link`. Its calls through `part` (the frames through the game). It
//! waits for a key: ungated. Shadow mode runs the original alone. Self-test: a copy whose
//! calls go to stand-ins (a key after some frames), against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, ZERO_FLAG};
use crate::net::lobby::{Part, RUN_FRAME, SLEEP, TAKE_KEY, game, lobby_entry};

const MESSAGE_FRAME: u32 = 0x41748D; // net::message_frame

/// The Rust version: the registers it leaves (the entry's).
pub fn show_message(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.esi = r.eax;
    r.eax = game(MESSAGE_FRAME);
    part(RUN_FRAME, &mut r)?;
    r.eax = 250;
    part(SLEEP, &mut r)?;
    part(TAKE_KEY, &mut r)?;
    loop {
        r.eax = game(MESSAGE_FRAME);
        part(RUN_FRAME, &mut r)?;
        part(TAKE_KEY, &mut r)?;
        if r.eflags & ZERO_FLAG == 0 {
            break;
        }
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, show_message)
}
