//! `sound::car_sounds_all` (0x40D7AB): the turn's sounds of every car: the players' cars
//! each, and of the computer's only the two nearest the listener.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! Unless `dword_514F7C` (the objects in use) is 0: for each object whose +8 is 1, a
//! player's (+0 not negative) gets `sound::car_sounds` (eax its number) at once; of the
//! others the squared distance (+0x2C .. +0x34 squared and added, through `flt_514F88`)
//! is compared as an integer (signed, the bits of a float that is not negative) with
//! the nearest and the next (both starting at the bits of +inf), which keep their
//! numbers in eax and ebx (-1 for none). When the nearest is last turn's second
//! (`dword_514F84`) or the next last turn's nearest (`dword_514F80`), the two swap, so
//! that each keeps its voice; they are kept in the two dwords and each gets
//! `sound::car_sounds` with its +0 set to -2 (the nearest) or -1 for the call and put
//! back after. One caller, `sound::race_turn`.
//! Refuses (before anything) a count over 8 and a null object; what the callees refuse
//! puts back what was written (sounds already sent stay sent).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::physics::flow::x87_run;
use crate::sound::store::ObjectsSlot;
use crate::sound::turn_parts::{COUNT, OBJECTS};

/// dword_514F80, dword_514F84: the nearest and the next of the computer's cars.
const NEAREST: ObjectsSlot = ObjectsSlot::at(0x514F80);
const NEXT: ObjectsSlot = ObjectsSlot::at(0x514F84);
/// flt_514F88: the squared distance.
const DISTANCE: ObjectsSlot = ObjectsSlot::at(0x514F88);

fn g(x: impl crate::store::Ida) -> u32 {
    unsafe { crate::store::ida_read(x) }
}

fn word(o: u32, at: u32) -> u32 {
    unsafe { ((o + at) as *const u32).read_unaligned() }
}

// ---- Rust version ----

/// Every car's sounds for the turn: the players' each, of the computer's the two nearest
/// the listener (each keeping its voice).
pub fn car_sounds_all() -> Result<(), Refused> {
    let count = g(COUNT);
    if count == 0 {
        return Ok(());
    }
    if count > 8 {
        return Err(BadInput("a sound object count over 8").into());
    }
    let object = |n: u32| g(OBJECTS + 4 * n as usize);
    if (0..count).any(|n| object(n) == 0) {
        return Err(BadInput("a null sound object").into());
    }
    // Converted: `rc2k_core::sound::turn::Nearest` (the choice; `car_sounds` keeps the
    // registers it is held in).
    use rc2k_core::sound::turn::{NEAREST_MARK, NEXT_MARK, Nearest};
    let mut nearest = Nearest::default();
    for n in 0..count {
        let o = object(n);
        if word(o, 8) != 1 {
            continue;
        }
        if word(o, 0) as i32 >= 0 {
            crate::sound::car_sounds::car_sounds(n)?;
            continue;
        }
        x87_run!(
                "fld dword ptr [{si} + 0x2C]",
                "fmul dword ptr [{si} + 0x2C]",
                "fld dword ptr [{si} + 0x30]",
                "fmul dword ptr [{si} + 0x30]",
                "fld dword ptr [{si} + 0x34]",
                "fmul dword ptr [{si} + 0x34]",
                ".byte 0xDE, 0xC1",
                ".byte 0xDE, 0xC1",
                "fstp dword ptr [{df} + 0x2578]";
            si = o, df = crate::sound::store::objects::base()
        );
        nearest.consider(n, g(DISTANCE));
    }
    let (first, second) = nearest.kept(g(NEAREST), g(NEXT));
    unsafe {
        crate::store::game_write(NEAREST, first);
        crate::store::game_write(NEXT, second);
    }
    for (which, mark) in [(NEAREST, NEAREST_MARK), (NEXT, NEXT_MARK)] {
        let n = g(which);
        if (n as i32) < 0 {
            continue;
        }
        let o = object(n);
        let kept = word(o, 0);
        unsafe { (o as *mut u32).write_unaligned(mark) };
        crate::sound::car_sounds::car_sounds(n)?;
        unsafe { (o as *mut u32).write_unaligned(kept) };
    }
    Ok(())
}
