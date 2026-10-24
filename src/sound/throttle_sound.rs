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
//! Refuses (before anything) a null object or request, and a kind wanted but null.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::physics::flow::x87_run;
use rc2k_core::sound::engine::{computers, revs_stopped};

// ---- Rust version ----

/// A player's engine sound by the throttle for object `o` of sound kind `kind`, through
/// `request`.
pub fn throttle_sound(o: u32, kind: u32, request: u32) -> Result<(), Refused> {
    if o == 0 || request == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let w = |at: u32| unsafe { ((o + at) as *const u32).read_unaligned() };
    let q = request as *mut u32;
    // Converted: `rc2k_core::sound::engine` (`computers`, `revs_stopped`).
    if computers(w(0)) {
        return Ok(());
    }
    if kind == 0 {
        return Err(BadInput("no sound kind").into());
    }
    let set = kind + 0x38;
    let sound = w(0).wrapping_add(unsafe { (set as *const u32).read_unaligned() });
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
                "fld dword ptr [{bx} + 0x104]",
                "fmul dword ptr [{bp} + 0x24]",
                "fadd dword ptr [{bp} + 0x1C]",
                "fmul dword ptr [{sf} + 0x28]",
                "fstp dword ptr [{si} + 0x8]";
            bx = o, bp = set, si = request, sf = crate::sound::store::surface::base()
        );
    } else {
        x87_run!(
            "fld dword ptr [{bx} + 0x104]",
            "fmul dword ptr [{bp} + 0x24]",
            "fadd dword ptr [{bp} + 0x1C]",
            "fstp dword ptr [{si} + 0x8]";
            bx = o, bp = set, si = request
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
        bx = o, si = request
    );
    crate::sound::turn_parts::placed(request)
}
