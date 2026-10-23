//! `net::message_frame` (0x41748D): the frame of `net::show_message`: the surface locked
//! (`gfx::lock_surface`; failing, nothing), the background copied, the message (string esi)
//! drawn with `font_b` at 0x190, 0x12C, unlocked and shown.
//!
//! In:  esi = the message's string id
//! Out: every register kept (`pusha` .. `popa`)
//! Called by `screens::run_frame` for `net::show_message`. Its calls through `part` (the
//! frame's platform through the game). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose calls go to stand-ins, against the Rust version, the lock failing
//! or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    COPY_PICTURE, DRAW_STRING, LOCK, Part, SHOW_FRAME, UNLOCK, a, lobby_entry,
};

const FONT_B: usize = 0x51E844; // font_b

/// A frame with one string: locked, the background, the string, unlocked, shown; nothing if
/// the lock fails. The registers as it leaves them.
pub fn string_frame(
    r: &mut Regs,
    id: u32,
    font: usize,
    x: u32,
    y: u32,
    part: Part,
) -> Result<(), Refused> {
    part(LOCK, r)?;
    if r.carry() {
        return Ok(());
    }
    string_frame_locked(r, id, font, x, y, part)
}

/// `string_frame` after the lock.
pub fn string_frame_locked(
    r: &mut Regs,
    id: u32,
    font: usize,
    x: u32,
    y: u32,
    part: Part,
) -> Result<(), Refused> {
    part(COPY_PICTURE, r)?;
    r.eax = id;
    r.ebx = a(font);
    r.ecx = x;
    r.edx = y;
    part(DRAW_STRING, r)?;
    part(UNLOCK, r)?;
    part(SHOW_FRAME, r)
}

/// The Rust version: the registers it leaves (the entry's).
pub fn message_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    string_frame(&mut r, entry.esi, FONT_B, 0x190, 0x12C, part)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, message_frame)
}
