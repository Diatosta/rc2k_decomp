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

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87_run::{ext_times, ints_diff_80};
use crate::scene::matrix3::{extended, rounded, test_floats};
use crate::scene::rotate_point::rotate_point;
use crate::scene::vector3::{read_floats, write_floats};
use crate::screens::harness::Left;
use crate::service::floats::{
    FLOAT, Floats, SUM, Tolerance, at_test_precisions, check_state_floats, dispatch_state_floats,
};
use crate::service::harness::{Region, capture, restore};
use rc2k_core::physics::sound_places as places;
use rc2k_core::physics::x87::loaded;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::car_sound_state", rva(0x45CACE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
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
    let b = game_addr(0) as u32;
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x31A260]",
            "fdiv dword ptr [{b} + 0x213EE0]",
            "fstp dword ptr [{b} + 0x31A264]",
            ".byte 0xD9, 0xE8", // fld1
            "fstp dword ptr [{b} + 0x31A268]",
            b = in(reg) b,
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
            "fld dword ptr [{b} + 0x31A268]",
            "fst dword ptr [{o} + 0x4C]",
            "fst dword ptr [{o} + 0x60]",
            "fst dword ptr [{o} + 0x74]",
            "fstp dword ptr [{o} + 0x88]",
            "fld dword ptr [{b} + 0x31A264]",
            "fmul dword ptr [{b} + 0x1C5E2C]",
            "fst dword ptr [{o} + 0x50]",
            "fst dword ptr [{o} + 0x64]",
            "fst dword ptr [{o} + 0x78]",
            "fstp dword ptr [{o} + 0x8C]",
            "fld dword ptr [{b} + 0x31A26C]",
            "fst dword ptr [{o} + 0x44]",
            "fst dword ptr [{o} + 0x58]",
            "fst dword ptr [{o} + 0x6C]",
            "fstp dword ptr [{o} + 0x80]",
            b = in(reg) b,
            o = in(reg) o,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

fn object() -> u32 {
    g(0x512A10 + 4 * (g(0x71B3F8) & 7) as usize)
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

/// The converted version (`rc2k_core::physics::sound_places`, plain floats): the hook's.
// Converted: `rc2k_core::physics::sound_places::{voices, pitch, volume, place,
// turned_motion}`.
pub unsafe fn car_sound_state_plain() -> Result<Left, Refused> {
    let o = object();
    if o == 0 {
        return Err(BadInput("a null sound object").into());
    }
    let l = listener();
    if l == 0 {
        return Err(BadInput("a null view block").into());
    }
    let f = |x: usize| f32::from_bits(g(x));
    unsafe {
        put(o + 0x10, 1);
        put(o + 0x14, g(0x71A1BC));
        let w = g(0x71A908);
        for at in [0x54, 0x68, 0x7C, 0x90] {
            put(o + at, w);
        }
        let all_off = g(0x71A22C) != 0;
        let v = places::voices(f(0x71A260), f(0x613EE0), f(0x71A26C), all_off);
        p(0x71A264, v.share.to_bits());
        p(0x71A268, v.full.to_bits());
        if all_off {
            p(0x71A26C, 0);
        }
        let pitch = places::pitch(f(0x71A264), f(0x5C5E2C));
        let motion = loaded(f(0x71A26C));
        for k in 0..4u32 {
            put(o + 0x4C + 0x14 * k, loaded(f(0x71A268)).to_bits());
            put(o + 0x50 + 0x14 * k, pitch.to_bits());
            put(o + 0x44 + 0x14 * k, motion.to_bits());
        }
        put(o + 0x18, places::volume(g(0x71A610)));
        put(o + 0xE8, g(0x71A1AC));
        put(o + 0x1C, g(0x71A198));
        put(o + 0xAC, g(0x71A220) & g(0x71A224));
        put(o + 0x0C, g(0x71A6C8) ^ 1);
        let whole = |a: u32| [0, 4, 8].map(|k| get(a + k) as i32);
        let view: [f32; 15] = read_floats(game_addr(rva(0x5EA24C)) as u32);
        let place = places::place(
            whole(game_addr(rva(0x71A10C)) as u32),
            whole(l + 0x6A4),
            None,
            &view,
            f(0x5C5DA0),
        );
        write_floats(o + 0x2C, &place);
        let turned = places::turned_motion(&view, [f(0x71A0AC), f(0x71A0B0), f(0x71A0B4)]);
        write_floats(o + 0x38, &turned);
    }
    Ok(Left::nothing())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { car_sound_state_plain() }
}

/// The float places: the shares kept, the voices' figures, the place and the motion.
fn floats_of(o: u32) -> Vec<(Region, Tolerance)> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![
        ((a(0x71A264), 4), FLOAT),
        ((o + 0x2C, 0x18), SUM),
        ((o + 0x44, 0x50), FLOAT),
    ]
}

fn regions() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut r = vec![(a(0x71A264), 12)];
    let o = object();
    if o != 0 {
        r.push((o, 0xF0));
    }
    r
}

extern "C" fn dispatch(r: &mut Regs) {
    let o = object();
    let memory = if o == 0 { vec![] } else { floats_of(o) };
    let floats = Floats {
        memory: &memory,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
}

/// What a sound test sets (`o` the test object).
pub fn test_regions(o: u32) -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![
        (o, 0x260),
        (a(0x512A10), 0x20),
        (a(0x71B3F8), 4),
        (a(0x63DB94), 4),
        (a(0x71A0AC), 12),
        (a(0x71A10C), 12),
        (a(0x71A67C), 12),
        (a(0x71A1AC), 4),
        (a(0x71A1BC), 4),
        (a(0x71A198), 4),
        (a(0x71A220), 16),
        (a(0x71A260), 16),
        (a(0x71A610), 4),
        (a(0x71A6C8), 4),
        (a(0x71A908), 4),
        (a(0x5EA24C), 0x40),
    ]
}

/// A sound case.
pub fn set_sound(o: u32, seed: u32) {
    unsafe {
        for k in 0..0x98u32 {
            put(o + 4 * k, 0xA5A5_0000 | k);
        }
    }
    for k in 0..8usize {
        p(0x512A10 + 4 * k, o);
    }
    p(0x71B3F8, seed % 8);
    p(0x63DB94, 0);
    let mut f = [0u32; 24];
    test_floats(seed * 11 + 7, &mut f);
    for (k, v) in f.iter().take(16).enumerate() {
        p(0x5EA24C + 4 * k, *v);
    }
    for k in 0..3usize {
        p(0x71A0AC + 4 * k, f[16 + k]);
        p(0x71A10C + 4 * k, seed.wrapping_mul(0x0246_8ACE) ^ k as u32);
        p(0x71A67C + 4 * k, (seed + 5).wrapping_mul(0x1357_9BDF) >> 3);
    }
    p(0x71A1AC, seed % 4);
    p(0x71A1BC, 3000 + seed * 100);
    p(0x71A198, f[19]);
    p(0x71A220, seed & 1);
    p(0x71A224, (seed >> 1) & 1);
    p(0x71A22C, (seed % 3 == 2) as u32);
    p(0x71A260, f[20]);
    p(0x71A26C, f[21]);
    p(
        0x71A610,
        [0u32, 0x80, 0x200, 0xFFFF_FFF0][(seed % 4) as usize],
    );
    p(0x71A6C8, seed % 2);
    p(0x71A908, seed * 3);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut object = vec![0u32; 0x98];
    let o = object.as_mut_ptr() as u32;
    let all = test_regions(o);
    let saved = capture(&all);
    let memory = floats_of(o);
    let tolerant = Floats {
        memory: &memory,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, floats| {
        for seed in 0..6u32 {
            check_state_floats(
                &mut t,
                "physics::car_sound_state",
                &format!("control word {word:#x}, seed {seed}"),
                call_original,
                Regs::SENTINEL,
                &|| set_sound(o, seed),
                &all,
                floats,
                &rust,
            );
        }
    });
    restore(&all, &saved);
    drop(object);
    t.finish()
}
