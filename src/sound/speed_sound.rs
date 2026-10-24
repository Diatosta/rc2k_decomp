//! `sound::speed_sound` (0x40DE09): a player's car's sound by its speed (+0x108, from
//! `sound::car_engine_state`), at the car.
//!
//! In:  ebx the car's sound object, edi its sound kind, esi the request
//! Out: eax the object's +0 (+ set +0 when stopped; +0x34 when placed), ebp the set
//!      (kind +0x60); everything else preserved
//! Nothing for a computer's car (+0 negative). The sound: kind +0x60's +0 + the
//! object's +0, into the request. With the state (+0x10C) 0 or 1, +0x24 1 (no gear),
//! +0xF4 or +0xAC not 0: `sound::stop_sound`. Else the request's +4 = +0x108 x set
//! +0x18 + set +0x10, +8 = +0x108 x set +0x24 + set +0x1C (x `flt_51637C` when +0xC is
//! 0), the place = +0x2C .. +0x34, and `sound::play_placed_sound`. Both are tail jumps.
//! One caller, `sound::car_sounds`.
//! Refuses (before anything) a null object or request, and a kind wanted but null.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::physics::flow::x87_run;
use rc2k_core::sound::engine::{computers, speed_stopped};

// ---- Rust version ----

/// A player's sound by the speed for object `o` of sound kind `kind`, through `request`.
pub fn speed_sound(o: u32, kind: u32, request: u32) -> Result<(), Refused> {
    if o == 0 || request == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let w = |at: u32| unsafe { ((o + at) as *const u32).read_unaligned() };
    let q = request as *mut u32;
    // Converted: `rc2k_core::sound::engine` (`computers`, `speed_stopped`).
    if computers(w(0)) {
        return Ok(());
    }
    if kind == 0 {
        return Err(BadInput("no sound kind").into());
    }
    let set = kind + 0x60;
    let sound = w(0).wrapping_add(unsafe { (set as *const u32).read_unaligned() });
    unsafe { q.write_unaligned(sound) };
    if speed_stopped(w(0x10C), || w(0x24), || w(0xF4), || w(0xAC)) {
        return crate::sound::turn_parts::stop(request);
    }
    x87_run!(
        "fld dword ptr [{bx} + 0x108]",
        "fmul dword ptr [{bp} + 0x18]",
        "fadd dword ptr [{bp} + 0x10]",
        "fstp dword ptr [{si} + 0x4]";
        bx = o, bp = set, si = request
    );
    if w(0xC) == 0 {
        x87_run!(
                "fld dword ptr [{bx} + 0x108]",
                "fmul dword ptr [{bp} + 0x24]",
                "fadd dword ptr [{bp} + 0x1C]",
                "fmul dword ptr [{sf} + 0x24]",
                "fstp dword ptr [{si} + 0x8]";
            bx = o, bp = set, si = request, sf = crate::sound::store::surface::base()
        );
    } else {
        x87_run!(
            "fld dword ptr [{bx} + 0x108]",
            "fmul dword ptr [{bp} + 0x24]",
            "fadd dword ptr [{bp} + 0x1C]",
            "fstp dword ptr [{si} + 0x8]";
            bx = o, bp = set, si = request
        );
    }
    for k in 0..3 {
        unsafe { q.add(3 + k).write_unaligned(w(0x2C + 4 * k as u32)) };
    }
    crate::sound::turn_parts::placed(request)
}
