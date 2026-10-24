//! `marks::emit_bursts` (0x4CD21A): two bursts of ring entries from a wheel (kinds 0x11
//! and 0x10), when the wheel's second flags (+0x24, kept in `dword_939BAA`) >> 6 are not
//! 0. Each half: a strength (`flt_935694`: the wheel's +0x10 times a factor; the second
//! `flt_935698`: a blend of +0xC and +0x18) and its rate; when the half's random's bits
//! 16 .. 23 are under the rate, a direction (`flt_935688` ..) and a count (a random's low
//! 12 bits modulo (flags >> 6) times 4); then that many entries: their motion the
//! direction spread (`marks::spread_point`), a size, a look (+0x58), a turn from the car's
//! +0x4C .. (+0x38 ..), a place at the wheel less the car's matrix's place plus a random
//! side, a fade (+0x5C), a life of 2000, +0x7C / +0x80 `dword_935476` / `dword_93547A`.
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! Callers: `marks::emit_wheel_particles`' kinds 2 .. 4, 6 .. 9, 0xC. The original's loops
//! push esi; when the ring is full (0x3FE taken) inside one they leave by 0x4CD741, which
//! takes only that push off: `popa` restores shifted registers and `retn` returns to a
//! register's value, the crash over mud (section 16; seen by the owner 2026-10-23). Fixed
//! here (2026-10-23): the bursts end at a full ring, as the exit with `lea esp, [esp+8]`
//! does. Refuses (before anything) a null wheel, a null object, car or matrix where read.

use crate::errors::{BadInput, Refused};
use crate::marks::emitters::*;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};

const FLAGS2_RVA: Slot = Slot::at(0x939BAA); // g_marksWheelFlags2: the wheel's +0x24 kept
const DIRECTION_RVA: Slot = Slot::at(0x935688); // flt_935688 .. flt_935690
const SCRATCH_RVA: Slot = Slot::at(0x765138); // dword_765138
const PICK_RVA: Slot = Slot::at(0x9356C0); // dword_9356C0
const RATE_SCALE_RVA: Slot = Slot::at(0x765190); // flt_765190
const HALF_K_RVA: Slot = Slot::at(0x76514C); // flt_76514C
const SPREAD_RVA: Slot = Slot::at(0x765158); // flt_765158
const TURN_K_RVA: Slot = Slot::at(0x765150); // flt_765150
const FADE_K_RVA: Slot = Slot::at(0x765214); // flt_765214
const LOOKS_RVA: [Slot; 2] = [Slot::at(0x935476), Slot::at(0x93547A)];
const LIFE: u32 = 0x7D0;

/// One half's variables.
pub struct Half {
    pub kind: u32,
    pub strength: Slot,
    pub rate_random: Slot,
    pub count_random: Slot,
    pub entry_random: Slot,
    /// The direction's factors (x, y, z) and whether |s| is scaled by `flt_765158` in y.
    pub factors: [Slot; 3],
    pub scaled: bool,
    /// The size's mask and base, the side's factors.
    pub size: (u32, u32),
    pub sides: [Slot; 3],
}

/// One function's variables: its two halves, the strength's factor, the blend's two.
pub struct BurstSet {
    pub halves: [Half; 2],
    pub strength_k: Slot,
    pub blend_k: [Slot; 2],
}

/// `emit_bursts`'.
pub const FIRST: BurstSet = BurstSet {
    halves: [
        Half {
            kind: 0x11,
            strength: Slot::at(0x935694),
            rate_random: Slot::at(0x93A3D6),
            count_random: Slot::at(0x93A3DA),
            entry_random: Slot::at(0x93A3DE),
            factors: [Slot::at(0x93545E), Slot::at(0x935462), Slot::at(0x935466)],
            scaled: false,
            size: (0x7FF, 0x400),
            sides: [Slot::at(0x93543A), Slot::at(0x93543E), Slot::at(0x935442)],
        },
        Half {
            kind: 0x10,
            strength: Slot::at(0x935698),
            rate_random: Slot::at(0x93A3EA),
            count_random: Slot::at(0x93A3EE),
            entry_random: Slot::at(0x93A3F2),
            factors: [Slot::at(0x93543A), Slot::at(0x93543E), Slot::at(0x935442)],
            scaled: true,
            size: (0x3FF, 0x600),
            sides: [Slot::at(0x93545E), Slot::at(0x935462), Slot::at(0x935466)],
        },
    ],
    strength_k: Slot::at(0x93A3D2),
    blend_k: [Slot::at(0x93A3E2), Slot::at(0x93A3E6)],
};

/// The half's strength and rate (`n` 0 or 1), from the wheel.
unsafe fn strength(set: &BurstSet, n: usize, wheel: u32) -> (u32, i32) {
    unsafe {
        let scale: u32 = game_read(RATE_SCALE_RVA);
        if n == 0 {
            x87::times_kept_abs_to_int(at(wheel + 0x10), game_read(set.strength_k), scale)
        } else {
            x87::blend_kept_abs_to_int(
                at(wheel + 0xC),
                at(wheel + 0x18),
                game_read(set.blend_k[0]),
                game_read(set.blend_k[1]),
                scale,
            )
        }
    }
}

/// The entries each half would make, worked out without writing.
unsafe fn counts(set: &BurstSet, wheel: u32) -> [u32; 2] {
    unsafe {
        let flags2 = at(wheel + 0x24);
        if flags2 >> 6 == 0 {
            return [0; 2];
        }
        std::array::from_fn(|n| {
            let h = &set.halves[n];
            let (_, rate) = strength(set, n, wheel);
            let r = game_read::<u32>(h.rate_random)
                .wrapping_mul(0xC5)
                .wrapping_add(0x5D);
            if (r.rotate_right(16) & 0xFF) as i32 >= rate {
                return 0;
            }
            let c = game_read::<u32>(h.count_random)
                .wrapping_mul(0xC5)
                .wrapping_add(0x5D);
            (c & 0xFFF) % ((flags2 >> 6) << 2)
        })
    }
}

/// Why the Rust version refuses, if it does, worked out without writing.
pub unsafe fn refusal(set: &BurstSet, wheel: u32, object: u32) -> Option<&'static str> {
    unsafe {
        if wheel == 0 {
            return Some("a null wheel");
        }
        if at(wheel + 0x24) >> 6 == 0 {
            return None;
        }
        let [a, b] = counts(set, wheel);
        let car: u32 = game_read(OBJECT_RVA);
        if a + b > 0 && (object == 0 || car == 0 || at(car + 0x24) == 0) {
            return Some("a null object, car or matrix");
        }
        None
    }
}

/// The Rust version (see the header).
pub unsafe fn emit_bursts(wheel: u32, object: u32) -> Result<(), Refused> {
    unsafe { emit_bursts_with(&FIRST, wheel, object) }
}

/// The shared body, on `set`'s variables.
pub unsafe fn emit_bursts_with(set: &BurstSet, wheel: u32, object: u32) -> Result<(), Refused> {
    unsafe {
        if let Some(why) = refusal(set, wheel, object) {
            return Err(BadInput(why).into());
        }
        let flags2 = at(wheel + 0x24);
        game_write(FLAGS2_RVA, flags2);
        if flags2 >> 6 == 0 {
            return Ok(());
        }
        for (n, h) in set.halves.iter().enumerate() {
            let (s, rate) = strength(set, n, wheel);
            game_write(h.strength, s);
            game_write(PICK_RVA, rate);
            let r = next_c5(h.rate_random);
            if (r.rotate_right(16) & 0xFF) as i32 >= rate {
                continue;
            }
            let car: u32 = game_read(OBJECT_RVA);
            let k: u32 = game_read(HALF_K_RVA);
            let s: u32 = game_read(h.strength);
            let scale = h.scaled.then(|| game_read::<u32>(SPREAD_RVA));
            let direction = [
                x87::mul2(game_read(h.factors[0]), s, k),
                x87::times_less_abs_times(game_read(h.factors[1]), s, scale, k),
                x87::mul2(game_read(h.factors[2]), s, k),
            ];
            game_write(DIRECTION_RVA, direction);
            let count = (next_c5(h.count_random) & 0xFFF) % ((flags2 >> 6) << 2);
            for _ in 0..count {
                // The ring full (0x3FE taken): the original leaves by 0x4CD741 / 0x4CDC6E,
                // whose stack slip crashed the game (section 16). Fixed 2026-10-23: the
                // bursts end there, the caller's registers kept, as `lea esp, [esp+8]`
                // would leave them.
                if game_read::<u32>(RING_COUNT_RVA) == MOST_TAKEN {
                    return Ok(());
                }
                let e = take_entry(at(object + 0x1C) & 0xFF_FFFF, h.kind);
                let put = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
                let d: [u32; 3] = game_read(DIRECTION_RVA);
                let (out, _) = crate::marks::spread_point::spread_point([
                    extended(game_read(SPREAD_RVA)),
                    extended(d[2]),
                    extended(d[1]),
                    extended(d[0]),
                ]);
                put(0x34, rounded(out[0]));
                put(0x30, rounded(out[1]));
                put(0x2C, rounded(out[2]));
                let r = next_c5(h.entry_random);
                let size = (r.rotate_right(8) & h.size.0) + h.size.1;
                game_write(SCRATCH_RVA, size);
                put(0x60, x87::from_int(size as i32));
                put(0x58, ((flags2 & 0x3F) << 4) + (r.rotate_right(17) & 3));
                let tk: u32 = game_read(TURN_K_RVA);
                put(0x40, x87::mul(at(car + 0x54), tk));
                put(0x3C, x87::mul(at(car + 0x50), tk));
                put(0x38, x87::mul(at(car + 0x4C), tk));
                for o in [0x44, 0x48, 0x4C] {
                    put(o, 0);
                }
                let m = at(car + 0x24);
                let side = (r.rotate_right(8) & 0x3FFF).wrapping_sub(0x2000);
                game_write(SCRATCH_RVA, side);
                let place: [u32; 3] = std::array::from_fn(|k| {
                    let k4 = 4 * k as u32;
                    x87::less_plus_int_times(
                        at(wheel + 0x3C + k4),
                        at(m + 0x30 + k4),
                        game_read(h.sides[k]),
                        side as i32,
                    )
                });
                put(0x10, place[0]);
                put(0x14, place[1]);
                put(0x28, place[1]);
                put(0x18, place[2]);
                let pick = r & 0x7_FFFF;
                game_write(PICK_RVA, pick);
                put(
                    0x5C,
                    x87::int_times_plus(pick as i32, game_read(FADE_K_RVA), game_read(ONE_RVA)),
                );
                put(0x84, LIFE);
                put(0x7C, game_read(LOOKS_RVA[0]));
                put(0x80, game_read(LOOKS_RVA[1]));
                put(0x68, car);
            }
        }
    }
    Ok(())
}
