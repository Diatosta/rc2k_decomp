//! What the marks' particle emitters under `sub_4CA085` share (IDA, 2026-10-17): the ring
//! of 0x94-byte entries (`g_marksRingCursor`, `g_marksRingCount`, at most 0x3FE taken),
//! the rate gate on a wheel record (edi) and its object (esi), and their randoms.
//! `marks::emit_wheel_particles` (0x4CA085) picks an emitter by the wheel record's +0x1C
//! (the surface's particle kind); the patched EXE some owners use turns it into a `ret`
//! (section 16).

use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;

pub use crate::marks::set_up_surface::{RING_COUNT_RVA, RING_CURSOR_RVA};

pub const OBJECT_RVA: Slot = Slot::at(0x935436); // g_marksCar: the car object being done
pub const TAG_RVA: Slot = Slot::at(0x93547E); // dword_93547E: each entry's +8
pub const FLAGS_RVA: Slot = Slot::at(0x939BAE); // g_marksWheelFlags: the wheel's +0x20 kept
pub const RATE_RVA: Slot = Slot::at(0x9356B0); // dword_9356B0
pub const SIDES_RVA: Slot = Slot::at(0x93554A); // dword_93554A, dword_93554E
pub const ONE_RVA: Slot = Slot::at(0x76516C); // flt_76516C
pub const MOST_TAKEN: u32 = 0x3FE;

/// A random of the emitters' kind: times 0xC5 plus 0x5D, kept.
pub unsafe fn next_c5(at: Slot) -> u32 {
    unsafe {
        let r = game_read::<u32>(at).wrapping_mul(0xC5).wrapping_add(0x5D);
        game_write(at, r);
        r
    }
}

/// The entry the next `take_entry` takes, if one is left.
pub unsafe fn next_entry() -> Option<u32> {
    unsafe {
        if game_read::<u32>(RING_COUNT_RVA) == MOST_TAKEN {
            return None;
        }
        Some((game_read::<u32>(RING_CURSOR_RVA) as *const u32).read_unaligned())
    }
}

/// The next entry taken (its +8 `dword_93547E`, +0x90 `tag`, +0xC `kind`).
pub unsafe fn take_entry(tag: u32, kind: u32) -> u32 {
    unsafe {
        let e = (game_read::<u32>(RING_CURSOR_RVA) as *const u32).read_unaligned();
        game_write(RING_CURSOR_RVA, e);
        game_write(
            RING_COUNT_RVA,
            game_read::<u32>(RING_COUNT_RVA).wrapping_add(1),
        );
        let put = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
        put(8, game_read(TAG_RVA));
        put(0x90, tag);
        put(0xC, kind);
        e
    }
}

/// Reads a dword at an address.
pub unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The rate gate's outcome, worked out without writing.
pub struct Gate {
    /// `dword_9356B0`.
    pub rate: i32,
    /// The wheel's new +4 when the second test is reached.
    pub count: Option<u32>,
    /// Whether an entry is made (all tests passed and the wheel moves).
    pub passes: bool,
}

/// The gate: the rate (`x87::rate_to_int` of the wheel's `a` and `b` floats by `k`); the
/// object's +0x90 plus it at least 0x1000; the wheel's +4 plus the object's +0x88 at
/// least 1.5 times that sum (both signed), then kept as its remainder by it; the wheel's
/// +0x10 or +0x18 not 0.
pub unsafe fn gate(wheel: u32, object: u32, a: u32, b: u32, k: [Slot; 4]) -> Gate {
    unsafe {
        let [k1, k2, k3, k4] = k.map(|r| game_read::<u32>(r));
        let rate = x87::rate_to_int(
            at(wheel + a),
            at(wheel + b),
            k1,
            k2,
            k3,
            game_read(ONE_RVA),
            k4,
        );
        gate_with(wheel, object, rate)
    }
}

/// The gate after the rate (see `gate`).
pub unsafe fn gate_with(wheel: u32, object: u32, rate: i32) -> Gate {
    unsafe {
        let sum = at(object + 0x90).wrapping_add(rate as u32);
        if (sum as i32) < 0x1000 {
            return Gate {
                rate,
                count: None,
                passes: false,
            };
        }
        let total = at(wheel + 4).wrapping_add(at(object + 0x88));
        let limit = sum.wrapping_add(sum).wrapping_sub(sum >> 1);
        if (total as i32) < limit as i32 {
            return Gate {
                rate,
                count: Some(total),
                passes: false,
            };
        }
        // `xor edx, edx; idiv`: the dividend the unsigned total.
        let rest = (total as i64 % limit as i32 as i64) as u32;
        let moving = at(wheel + 0x10) != 0 || at(wheel + 0x18) != 0;
        Gate {
            rate,
            count: Some(rest),
            passes: moving,
        }
    }
}
