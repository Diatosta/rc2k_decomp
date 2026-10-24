//! `physics::car_sound_state` (0x45CACE): the work car's engine sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`) told the car's state: +0x10 1; +0x14 the revs
//! (+0x1E4); +0x54, +0x68, +0x7C, +0x90 `g_car` +0x930; the share +0x28C =
//! +0x288 / `flt_613EE0` and +0x290 1.0 (with +0x254 set, those and +0x294 0), into the
//! object's four voices (+0x4C .., x256 at +0x50 .., +0x294 at +0x44 ..); +0x18 +0x638
//! held to 0 ..= 0x100; +0xE8 the gear; +0x1C +0x1C0; +0xAC +0x248 & +0x24C; +0x0C
//! +0x6F0 ^ 1; the car's place less the listener's (`g_viewBlocks[0]`'s with
//! `dword_63DB94` 1, else the work car's own +0x6A4 ..) turned by `g_viewTurnMatrix`, x
//! 1/65536 into +0x2C ..; the car's +0xD4 .. turned into +0x38 ... All in `pusha` /
//! `popa`.
//!
//! The hook runs the converted version (`car_sound_state_plain`); the exact x87 one
//! (the runs over the object the original's instructions, from IDA) stays for
//! `physics::engine_sound`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `physics::engine_sound`.
//! Refuses (before anything) a null sound object, or a null view block when it is read.
//! Self-test: the original and the converted version on the game's variables (saved and
//! put back) with a test object: exact at double precision; at extended the shares and
//! voices within `FLOAT`, the place and the turned motion within `SUM`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_write, rva};
use crate::math::x87_run::{ext_times, ints_diff_80};
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;

fn g(x: impl crate::store::At) -> u32 {
    unsafe { crate::store::at_read(x) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// 0x45CB02 .. 0x45CB88 (the shares into the voices), the integer steps between in Rust.
unsafe fn shares(o: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{rc} + 0x31A260]",
            "fdiv dword ptr [{rc} + 0x213EE0]",
            "fstp dword ptr [{rc} + 0x31A264]",
            ".byte 0xD9, 0xE8", // fld1
            "fstp dword ptr [{rc} + 0x31A268]",
            rc = in(reg) crate::race::store::asm_base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    if g(0x71A22C) != 0 {
        p(0x71A264, 0);
        p(0x71A268, 0);
        p(0x71A26C, 0);
    }
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{rc} + 0x31A268]",
            "fst dword ptr [{o} + 0x4C]",
            "fst dword ptr [{o} + 0x60]",
            "fst dword ptr [{o} + 0x74]",
            "fstp dword ptr [{o} + 0x88]",
            "fld dword ptr [{rc} + 0x31A264]",
            "fmul dword ptr [{rc} + 0x1C5E2C]",
            "fst dword ptr [{o} + 0x50]",
            "fst dword ptr [{o} + 0x64]",
            "fst dword ptr [{o} + 0x78]",
            "fstp dword ptr [{o} + 0x8C]",
            "fld dword ptr [{rc} + 0x31A26C]",
            "fst dword ptr [{o} + 0x44]",
            "fst dword ptr [{o} + 0x58]",
            "fst dword ptr [{o} + 0x6C]",
            "fstp dword ptr [{o} + 0x80]",
            rc = in(reg) crate::race::store::asm_base(),
            o = in(reg) o,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

fn object() -> u32 {
    g(crate::sound::store::ObjectsSlot::at(0x512A10) + 4 * (g(0x71B3F8) & 7) as usize)
}

fn listener() -> u32 {
    if g(0x63DB94) == 1 {
        g(0x63DBB0)
    } else {
        game_addr(rva(0x719FD8)) as u32
    }
}

/// The exact x87 version (for `physics::engine_sound`).
pub unsafe fn car_sound_state() -> Result<Left, Refused> {
    let o = object();
    if o == 0 {
        return Err(BadInput("a null sound object").into());
    }
    let l = listener();
    if l == 0 {
        return Err(BadInput("a null view block").into());
    }
    unsafe {
        put(o + 0x10, 1);
        put(o + 0x14, g(0x71A1BC));
        let w = g(0x71A908);
        for at in [0x54, 0x68, 0x7C, 0x90] {
            put(o + at, w);
        }
        shares(o);
        let n = g(0x71A610) as i32;
        put(o + 0x18, n.clamp(0, 0x100) as u32);
        put(o + 0xE8, g(0x71A1AC));
        put(o + 0x1C, g(0x71A198));
        put(o + 0xAC, g(0x71A220) & g(0x71A224));
        put(o + 0x0C, g(0x71A6C8) ^ 1);
        let d = |c: usize, k: u32| ints_diff_80(g(c) as i32, get(l + k) as i32);
        let place = [d(0x71A10C, 0x6A4), d(0x71A110, 0x6A8), d(0x71A114, 0x6AC)];
        let m = game_addr(rva(0x5EA24C)) as u32;
        let out = rotate_point(m, place);
        let k = g(0x5C5DA0);
        for (i, e) in out.iter().enumerate() {
            put(o + 0x2C + 4 * i as u32, rounded(ext_times(*e, k)));
        }
        let out = rotate_point(
            m,
            [
                extended(g(0x71A0AC)),
                extended(g(0x71A0B0)),
                extended(g(0x71A0B4)),
            ],
        );
        for (i, e) in out.iter().enumerate() {
            put(o + 0x38 + 4 * i as u32, rounded(*e));
        }
    }
    Ok(Left::nothing())
}
