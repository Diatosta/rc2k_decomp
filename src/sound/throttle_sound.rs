//! `sound::throttle_sound` (0x40DD96): a player's car's engine sound by its throttle,
//! placed behind the car.
//!
//! In:  ebx the car's sound object, edi its sound kind, esi the request
//! Out: eax the object's +0 (+ set +0 when sent), ebp the set (kind +0x38) when sent;
//!      everything else preserved
//! Nothing for a computer's car (+0 negative). The sound: kind +0x38's +0 + the
//! object's +0, into the request. With the state (+0x10C) 0 or 3: `sound::stop_sound`.
//! Else the request's +4 = +0x100 x set +0x18 + set +0x10, +8 = +0x104 x set +0x24 +
//! set +0x1C (x `flt_516380` when +0xC is 0), the place = +0x2C .. +0x34 less +0x38 ..
//! +0x40, and `sound::play_placed_sound`. Both are tail jumps. One caller,
//! `sound::car_sounds`.
//! A short jump in its first five bytes: private copy (0x73 bytes, the three jumps out
//! relocated), manual patch.
//! Refuses (before anything) a null object or request, and a kind wanted but null.
//! Shadow mode runs the original alone (it sends a sound).
//! Self-test: a copy whose jumps out go to stand-ins, against the Rust version
//! (`turn_parts::test_car_sound`).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::physics::flow::{base, x87_run};
use crate::sound::turn_parts::{PLACED, Part, STOP, turn_entry};
use rc2k_core::sound::engine::{computers, revs_stopped};

// ---- Rust version ----

pub fn throttle_sound_in(c: &mut Regs, part: Part) -> Result<(), Refused> {
    if c.ebx == 0 || c.esi == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let o = c.ebx;
    let w = |at: u32| unsafe { ((o + at) as *const u32).read_unaligned() };
    let q = c.esi as *mut u32;
    // Converted: `rc2k_core::sound::engine` (`computers`, `revs_stopped`).
    c.eax = w(0);
    if computers(c.eax) {
        return Ok(());
    }
    if c.edi == 0 {
        return Err(BadInput("no sound kind").into());
    }
    c.ebp = c.edi + 0x38;
    c.eax = c
        .eax
        .wrapping_add(unsafe { (c.ebp as *const u32).read_unaligned() });
    unsafe { q.write_unaligned(c.eax) };
    if revs_stopped(w(0x10C)) {
        return part(STOP, c);
    }
    x87_run!(
        "fld dword ptr [{bx} + 0x100]",
        "fmul dword ptr [{bp} + 0x18]",
        "fadd dword ptr [{bp} + 0x10]",
        "fstp dword ptr [{si} + 0x4]";
        bx = c.ebx, bp = c.ebp, si = c.esi
    );
    if w(0xC) == 0 {
        x87_run!(
            "fld dword ptr [{bx} + 0x104]",
            "fmul dword ptr [{bp} + 0x24]",
            "fadd dword ptr [{bp} + 0x1C]",
            "fmul dword ptr [{b} + 0x116380]",
            "fstp dword ptr [{si} + 0x8]";
            bx = c.ebx, bp = c.ebp, si = c.esi, b = base()
        );
    } else {
        x87_run!(
            "fld dword ptr [{bx} + 0x104]",
            "fmul dword ptr [{bp} + 0x24]",
            "fadd dword ptr [{bp} + 0x1C]",
            "fstp dword ptr [{si} + 0x8]";
            bx = c.ebx, bp = c.ebp, si = c.esi
        );
    }
    x87_run!(
        "fld dword ptr [{bx} + 0x2C]",
        "fsub dword ptr [{bx} + 0x38]",
        "fstp dword ptr [{si} + 0xC]",
        "fld dword ptr [{bx} + 0x30]",
        "fsub dword ptr [{bx} + 0x3C]",
        "fstp dword ptr [{si} + 0x10]",
        "fld dword ptr [{bx} + 0x34]",
        "fsub dword ptr [{bx} + 0x40]",
        "fstp dword ptr [{si} + 0x14]";
        bx = c.ebx, si = c.esi
    );
    part(PLACED, c)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, throttle_sound_in)
}
