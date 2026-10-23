//! `physics::probe_and_keep` (0x45DC12): `physics::probe_ground`, then its three
//! whole-number lists (x, y, z at 0x6D1E14, 0xC0 dwords) copied to 0x6D2C14 and the probe
//! points' faces (`dword_71ADC8`, 0x40 dwords) to 0x6D2F14 (`rep movsd`): by the look of
//! it the step's starting points kept (a reading).
//!
//! In:  nothing
//! Out: ecx 0, esi 0x71ADC8 + 0x100, edi 0x6D2F14 + 0x100; eax, ebx, edx as
//!      `probe_ground` leaves them (with no probe points: kept)
//!
//! Caller: `sub_43B635` (0x43B700), every physics step; next a call (no flags read).
//! Refuses what `probe_ground` refuses (nothing written).
//! Self-test: a copy of the original (its call to a copy of `probe_ground` on the test's
//! ground) against the Rust core, `probe_ground`'s cases.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, EDI, EDX, ESI, Regs};
use crate::physics::ground::*;
use crate::physics::probe_ground::probe_ground_at;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust core; `r` as the original leaves it.
pub unsafe fn probe_and_keep_at(g: &Ground, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        probe_ground_at(g, r)?;
        for (from, to, n) in [
            (0x6D1E14u32, 0x6D2C14u32, 0xC0u32),
            (0x71ADC8, 0x6D2F14, 0x40),
        ] {
            for k in 0..n {
                g.put(to + 4 * k, g.get(from + 4 * k));
            }
            r.esi = g.a(from) + 4 * n;
            r.edi = g.a(to) + 4 * n;
        }
        r.ecx = 0;
        Ok(())
    }
}

/// The Rust version.
pub unsafe fn probe_and_keep(r: &mut Regs) -> Result<(), Refused> {
    unsafe { probe_and_keep_at(&Ground::game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { probe_and_keep(&mut g) }?;
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ECX, g.ecx)
        .with(EDX, g.edx)
        .with(ESI, g.esi)
        .with(EDI, g.edi))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
