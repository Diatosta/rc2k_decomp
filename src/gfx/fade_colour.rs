//! `gfx::fade_colour` (0x44A4C5): a colour brought toward another: each of the three low
//! bytes s of ebx becomes s + ((t - s) x ebp >> 8) (`imul`, `sar`), held to 0 .. 0xFF, t
//! the byte of ecx in the same place; the top byte kept.
//!
//! In:  ebx = the colour, ecx = the colour faded to, ebp = how far (0x100 all the way)
//! Out: eax = ebx = the colour; ecx kept (turned twice); esi the third byte of the colour
//!      before; edx the high dword of the last `imul`; no flags read
//!
//! Callers: 0x425A23, `race::splash_mud` (0x44A403), 0x44CB15. Self-test: the original
//! and the Rust version on colours and amounts by lot and at the edges.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// One byte's step: (the byte, the high dword of the `imul`). Converted:
/// `rc2k_core::gfx::pixels::fade_step`.
use rc2k_core::gfx::pixels::fade_step as step;

/// The three low bytes of `from` brought toward `to` (their places' values, whole: the
/// mud's tint gives dwords), `amount` of 0x100; the registers it leaves on `entry`.
pub fn fade_bytes(entry: &Regs, from: u32, to: [u32; 3], amount: u32) -> Regs {
    let mut r = *entry;
    let [b0, b1, b2, b3] = from.to_le_bytes();
    let (r0, _) = step(b0.into(), to[0], amount);
    let (r1, _) = step(b1.into(), to[1], amount);
    let (r2, high) = step(b2.into(), to[2], amount);
    r.ebx = u32::from_le_bytes([r0 as u8, r1 as u8, r2 as u8, b3]);
    r.eax = r.ebx;
    r.esi = b2.into();
    r.edx = high;
    r
}

/// The Rust version (see the header): the registers it leaves.
pub fn fade_colour(entry: &Regs) -> Regs {
    let [t0, t1, t2, _] = entry.ecx.to_le_bytes();
    fade_bytes(
        entry,
        entry.ebx,
        [t0.into(), t1.into(), t2.into()],
        entry.ebp,
    )
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&fade_colour(r)))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
