//! `sound::surface_sound` (0x40E068): one of a car's sounds of what its wheels run on.
//!
//! In:  ebx the car's sound object, ebp the surface's sound set, esi the request
//! Out: eax the place's last dword (+0x34), ecx and edx the volume and rate
//!      (`sound::sample_settings` of the sample's own); everything else preserved
//! The set: +0 1 for a set of eight (+4 ..; the sound by the object's +0 & 3 and, 4
//! further for +0xC 1), else four (by +0 & 3). Its sound replaces the object's +0x144
//! (sets of eight) or +0x148; a changed one has the old one stopped first
//! (`sound::stop_sound`). Then the request: the sound, +4 the rate as a float, +8
//! `flt_516398` (`fld` / `fstp`), the place +0x2C .. +0x34, and
//! `sound::play_placed_sound` (a tail jump). Two callers, `sound::surface_sounds`.
//! Refuses (before anything) a null object, set or request, a set of eight with +0xC
//! over 1, and a sound past the sample records.
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose calls go to stand-ins (the one of `sample_settings` left),
//! against the Rust version, on an object and sets of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::math::x87::from_int;
use crate::physics::flow::{base, x87_run};
use crate::sound::sample_settings::{OWN, record, sample_settings};
use crate::sound::turn_parts::turn_entry;
use rc2k_core::sound::surfaces;

// ---- Rust version ----

pub fn surface_sound_in(c: &mut Regs) -> Result<(), Refused> {
    if c.ebx == 0 || c.ebp == 0 || c.esi == 0 {
        return Err(BadInput("no object, set or request").into());
    }
    let o = c.ebx;
    let w = |at: u32| unsafe { ((o + at) as *const u32).read_unaligned() };
    let set_word = |at: u32| (o + at) as *mut u32;
    let set = c.ebp as *const u32;
    let q = c.esi as *mut u32;
    // Converted: `rc2k_core::sound::surfaces::pick`.
    let eight = unsafe { set.read_unaligned() } == 1;
    let (n, slot) = surfaces::pick(eight, w(0), || {
        c.edx = w(0xC);
        c.edx
    })
    .map_err(BadInput::from)?;
    c.ecx = n;
    c.eax = unsafe { set.add(1 + c.ecx as usize).read_unaligned() };
    record(c.eax)?;
    if c.eax != w(slot) {
        let old = w(slot);
        unsafe {
            set_word(slot).write_unaligned(c.eax);
            q.write_unaligned(old);
        }
        c.eax = old;
        crate::sound::turn_parts::stop(c)?;
        c.eax = w(slot);
    }
    unsafe { q.write_unaligned(c.eax) };
    (c.ecx, c.edx) = sample_settings(c.eax, OWN, OWN)?;
    unsafe { q.add(1).write_unaligned(from_int(c.edx as i32)) };
    x87_run!(
        "fld dword ptr [{b} + 0x116398]",
        "fstp dword ptr [{si} + 0x8]";
        b = base(), si = c.esi
    );
    // `mov eax, [ebx+..]; mov [esi+..], eax`: eax is left at the last.
    for k in 0..3 {
        c.eax = w(0x2C + 4 * k as u32);
        unsafe { q.add(3 + k).write_unaligned(c.eax) };
    }
    crate::sound::turn_parts::placed(c)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, surface_sound_in)
}
