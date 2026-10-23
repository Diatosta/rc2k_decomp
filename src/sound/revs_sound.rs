//! `sound::revs_sound` (0x40DD19): a car's engine sound by its revs, placed ahead of the
//! car.
//!
//! In:  ebx the car's sound object, edi its sound kind, esi the request
//! Out: eax the sound, ebp its set (kind +0x10, or `unk_516230` for a computer's car);
//!      everything else preserved
//! The sound: the set's +0 + the object's +0 (a computer's car, +0 negative: + its
//! bit 0, from `unk_516230`), into the request. With the state (+0x10C) 0 or 3:
//! `sound::stop_sound`. Else the request's +4 (pitch) = +0x100 x set +0x18 + set +0x10,
//! +8 (volume) = +0x100 x set +0x24 + set +0x1C (x `flt_516380` when +0xC is 0), the
//! place = +0x2C .. +0x34 plus +0x38 .. +0x40, and `sound::play_placed_sound`. Both
//! are tail jumps. One caller, `sound::car_sounds`.
//! A short jump in its first five bytes: private copy (0x7D bytes, the three jumps out
//! relocated), manual patch.
//! Refuses (before anything) a null object or request, and a kind wanted but null.
//! Shadow mode runs the original alone (it sends a sound).
//! Self-test: a copy whose jumps out go to stand-ins, against the Rust version
//! (`turn_parts::test_car_sound`).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::game_addr;
use crate::framework::rva;
use crate::physics::flow::{base, x87_run};
use rc2k_core::sound::engine::{computers, revs_sound_number, revs_stopped};

/// unk_516230: the computer's cars' set.
const COMPUTERS: usize = 0x516230;

// ---- Rust version ----

/// The engine's sound for object `o` of sound kind `kind`, through `request`.
pub fn revs_sound(o: u32, kind: u32, request: u32) -> Result<(), Refused> {
    if o == 0 || request == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let w = |at: u32| unsafe { ((o + at) as *const u32).read_unaligned() };
    let q = request as *mut u32;
    // Converted: `rc2k_core::sound::engine` (`computers`, `revs_sound_number`,
    // `revs_stopped`).
    let object = w(0);
    let set = if computers(object) {
        game_addr(rva(COMPUTERS)) as u32
    } else {
        if kind == 0 {
            return Err(BadInput("no sound kind").into());
        }
        kind + 0x10
    };
    let sound = revs_sound_number(object, unsafe { (set as *const u32).read_unaligned() });
    unsafe { q.write_unaligned(sound) };
    if revs_stopped(w(0x10C)) {
        return crate::sound::turn_parts::stop(request);
    }
    x87_run!(
        "fld dword ptr [{bx} + 0x100]",
        "fmul dword ptr [{bp} + 0x18]",
        "fadd dword ptr [{bp} + 0x10]",
        "fstp dword ptr [{si} + 0x4]";
        bx = o, bp = set, si = request
    );
    if w(0xC) == 0 {
        x87_run!(
            "fld dword ptr [{bx} + 0x100]",
            "fmul dword ptr [{bp} + 0x24]",
            "fadd dword ptr [{bp} + 0x1C]",
            "fmul dword ptr [{b} + 0x116380]",
            "fstp dword ptr [{si} + 0x8]";
            bx = o, bp = set, si = request, b = base()
        );
    } else {
        x87_run!(
            "fld dword ptr [{bx} + 0x100]",
            "fmul dword ptr [{bp} + 0x24]",
            "fadd dword ptr [{bp} + 0x1C]",
            "fstp dword ptr [{si} + 0x8]";
            bx = o, bp = set, si = request
        );
    }
    x87_run!(
        "fld dword ptr [{bx} + 0x2C]",
        "fadd dword ptr [{bx} + 0x38]",
        "fstp dword ptr [{si} + 0xC]",
        "fld dword ptr [{bx} + 0x30]",
        "fadd dword ptr [{bx} + 0x3C]",
        "fstp dword ptr [{si} + 0x10]",
        "fld dword ptr [{bx} + 0x34]",
        "fadd dword ptr [{bx} + 0x40]",
        "fstp dword ptr [{si} + 0x14]";
        bx = o, si = request
    );
    crate::sound::turn_parts::placed(request)
}
