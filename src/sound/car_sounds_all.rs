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
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose calls go to a stand-in, the tables pointing at the test's
//! objects (put back), against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::physics::flow::{base, x87_run};
use crate::sound::turn_parts::{COUNT, OBJECTS, turn_entry};

/// dword_514F80, dword_514F84: the nearest and the next of the computer's cars.
const NEAREST: usize = 0x514F80;
const NEXT: usize = 0x514F84;
/// flt_514F88: the squared distance.
const DISTANCE: usize = 0x514F88;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn word(o: u32, at: u32) -> u32 {
    unsafe { ((o + at) as *const u32).read_unaligned() }
}

// ---- Rust version ----

pub fn car_sounds_all_in(r: &mut Regs) -> Result<(), Refused> {
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
    // Converted: `rc2k_core::sound::turn::Nearest` (the choice; eax, ebx, ecx and edx
    // hold it as the original's do).
    use rc2k_core::sound::turn::{NEAREST_MARK, NEXT_MARK, Nearest};
    let hold = |c: &mut Regs, n: &Nearest| {
        (c.eax, c.ebx, c.ecx, c.edx) = (n.first, n.second, n.first_distance, n.second_distance)
    };
    {
        let mut c = *r;
        let mut nearest = Nearest::default();
        hold(&mut c, &nearest);
        for n in 0..count {
            c.ebp = n;
            c.esi = object(n);
            if word(c.esi, 8) != 1 {
                continue;
            }
            if word(c.esi, 0) as i32 >= 0 {
                let kept = c.eax;
                c.eax = n;
                crate::sound::car_sounds::car_sounds_in(&mut c)?;
                c.eax = kept;
                // The choice goes on from the registers as the call left them.
                nearest = Nearest {
                    first: c.eax,
                    second: c.ebx,
                    first_distance: c.ecx,
                    second_distance: c.edx,
                };
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
                "fstp dword ptr [{b} + 0x114F88]";
                si = c.esi, b = base()
            );
            nearest.consider(n, g(DISTANCE));
            hold(&mut c, &nearest);
        }
        c.ebp = count;
        (c.eax, c.ebx) = nearest.kept(g(NEAREST), g(NEXT));
        unsafe {
            game_write(rva(NEAREST), c.eax);
            game_write(rva(NEXT), c.ebx);
        }
        for (which, mark) in [(NEAREST, NEAREST_MARK), (NEXT, NEXT_MARK)] {
            c.eax = g(which);
            if (c.eax as i32) < 0 {
                continue;
            }
            c.esi = object(c.eax);
            let kept = word(c.esi, 0);
            unsafe { (c.esi as *mut u32).write_unaligned(mark) };
            crate::sound::car_sounds::car_sounds_in(&mut c)?;
            unsafe { (c.esi as *mut u32).write_unaligned(kept) };
        }
        Ok(())
    }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    turn_entry(r, car_sounds_all_in)
}
