//! `drive::set_listener` (0x4565B7): the state the sounds are placed by, from the viewed
//! car (`g_carBlocks[dword_63DB9C]`), car block 0 (0x70F410 ..) and the view: its three
//! whole numbers +0x134 .. +0x13C as floats (0x5C5630 ..), `dword_5C2FD4` = (+0xABC >=
//! 0x10); the view's place negated and its turn's nine (0x5C2FA0 .. 0x5C2FCC); three of
//! 0x6415D0 .. (the first 0 without `dword_6238A8`) into 0x5C56D8 ..; six whole numbers of
//! the car's light record (`g_lightRecords` +0x40 .., by `race::car_file_number` of
//! `dword_70F410`) times 65536 over 1000 (two less `dword_613FC4`) into 0x608C60 ..; car
//! block 0's matrix (0x70FC2C) through `scene::multiply_matrices` with 0x70FC6C and the
//! view matrix into `g_madeObjectMatrix` (and a copy at 0x5EA0CC); the two points of
//! 0x608C60 .. turned by it, each with its x and then -x (`scene::rotate_point`,
//! `fistp`'d into 0x5C568C .. 0x5C56B8); the switches of 0x5C56BC .. 0x5C56E8 by
//! `dword_6415E0`, `dword_70FCEC`, `dword_70FCF0`; `dword_5C5680` from `dword_70F628`
//! (then cleared), `dword_5C5684`; `flt_5C5660` = `dword_70FA3C` / 2^29 + `flt_637644`;
//! two floats copied; with `dword_601024` set, `dword_5C5680` and `dword_5C5684` 0. By
//! the look of it the listener for the 3D sounds (a reading).
//!
//! In:  nothing
//! Out: eax the old `dword_70F628`, ebx 0x10000, ecx 0, edx 0, esi `g_madeObjectMatrix`,
//!      edi 0x5EA10C, ebp `g_madeObjectMatrix`; no flags read
//!
//! Caller: `drive::share_all_cars` (0x456484), second.
//! Refuses (before anything) a viewed car past the eight blocks or with a null block, a
//! car file number above
//! 0x21, and a light record figure whose times 65536 over 1000 leaves 32 bits (the
//! original's `idiv` traps).
//! Self-test: a copy of the original on a `physics::Mirror` of its fourteen ranges (its
//! calls kept: `car_file_number`, `multiply_matrices`, `rotate_point`), switches and
//! figures by lot, under three precisions; the refusals.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, EBP, EBX, ECX, EDI, EDX, ESI, RawHook, Regs, SelfTest, call_game, hook_asm, rva,
};
use crate::math::x87;
use crate::math::x87_run::{
    TEST_CONTROL_WORDS, ext_to_int, int_80, int_over_int_plus, with_control_word,
};
use crate::physics::mirror::{Mirror, Spec};
use crate::race::mud::car_file_number;
use crate::scene::matrix::multiply_matrices;
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::set_listener", rva(0x4565B7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Everything it reads or writes but the light records (IDA addresses, end exclusive).
pub const RANGES: [(u32, u32); 14] = [
    (0x5C2FA0, 0x5C2FD8),
    (0x5C5620, 0x5C56EC),
    (0x5EA0CC, 0x5EA34C),
    (0x601024, 0x601028),
    (0x606298, 0x6062A4),
    (0x608C60, 0x608C78),
    (0x613FC4, 0x613FC8),
    (0x6238A8, 0x6238AC),
    (0x62400C, 0x624010),
    (0x637644, 0x637648),
    (0x63DB9C, 0x63DBA0),
    (0x6415D0, 0x6415E4),
    (0x70F410, 0x710300),
    (0x71B3B8, 0x71B3D8),
];
pub const SPEC: Spec = (
    0x4565B7,
    0x3E5,
    &[0x8B, 0x1D, 0x9C, 0xDB, 0x63, 0x00],
    &[0x10E, 0x1B0, 0x1C7, 0x1F9, 0x222, 0x24D, 0x276],
    124,
);
const TURNED: u32 = 0x5EA10C; // g_turnedMatrix
const LIT: u32 = 0x5EA14C; // g_litMatrix
const VIEW_TURN: u32 = 0x5EA24C; // g_viewTurnMatrix
const VIEW: u32 = 0x5EA2CC; // g_viewMatrix
const MADE: u32 = 0x5EA30C; // g_madeObjectMatrix
const LIGHT_FIGURES: u32 = 0x5C90AC; // g_lightRecords + 0x40

/// The car file's six figures times 65536 over 1000, or the refusal (before anything).
unsafe fn figures(m: &Mirror) -> Result<[i32; 6], Refused> {
    unsafe {
        let viewed = m.get(0x63DB9C);
        if viewed > 7 {
            return Err(BadInput("a viewed car past the eight blocks").into());
        }
        if m.get(0x71B3B8 + 4 * viewed) == 0 {
            return Err(BadInput("the viewed car's block is null").into());
        }
        let n = car_file_number(m.get(0x70F410));
        if n > 0x21 {
            return Err(BadInput("a car file number above 0x21 (past g_lightRecords)").into());
        }
        let mut out = [0i32; 6];
        for (k, o) in out.iter_mut().enumerate() {
            let v = m.get(LIGHT_FIGURES + 0x134 * n + 4 * k as u32) as i32;
            let q = (i64::from(v) << 16) / 1000;
            if q != i64::from(q as i32) {
                return Err(BadInput(
                    "a light record figure whose times 65536 / 1000 leaves 32 bits",
                )
                .into());
            }
            *o = q as i32;
        }
        Ok(out)
    }
}

/// The Rust core on the variables `m` reaches.
pub unsafe fn set_listener_at(m: &Mirror) -> Result<Left, Refused> {
    unsafe {
        let f = figures(m)?;
        let g = |v: u32| m.get(v);
        let s = m.get(0x71B3B8 + 4 * m.get(0x63DB9C));
        let at = |o: u32| (s.wrapping_add(o) as *const u32).read_unaligned();
        m.put(0x5C5638, x87::from_int(at(0x13C) as i32));
        m.put(0x5C5634, x87::from_int(at(0x138) as i32));
        m.put(0x5C5630, x87::from_int(at(0x134) as i32));
        m.put(0x5C2FD4, u32::from(at(0xABC) >= 0x10));
        for k in 0..3 {
            m.put(0x5C2FA0 + 4 * k, g(0x606298 + 4 * k).wrapping_neg());
        }
        for (k, o) in [0u32, 4, 8, 0x10, 0x14, 0x18, 0x20, 0x24, 0x28]
            .into_iter()
            .enumerate()
        {
            m.put(0x5C2FAC + 4 * k as u32, g(VIEW_TURN + o));
        }
        m.put(0x5C56D8, if g(0x6238A8) == 0 { 0 } else { g(0x6415D8) });
        m.put(0x5C56DC, g(0x6415D0));
        m.put(0x5C56E0, g(0x6415D4));
        let less = g(0x613FC4);
        let f = [
            f[0],
            f[1].wrapping_sub(less as i32),
            f[2],
            f[3],
            f[4].wrapping_sub(less as i32),
            f[5],
        ];
        for (k, &v) in f.iter().enumerate() {
            m.put(0x608C60 + 4 * k as u32, v as u32);
        }
        for k in 0..16 {
            m.put(TURNED + 4 * k, g(0x70FC2C + 4 * k));
        }
        multiply_matrices(
            m.a(TURNED) as usize,
            m.a(0x70FC6C) as usize,
            m.a(LIT) as usize,
        );
        multiply_matrices(m.a(LIT) as usize, m.a(VIEW) as usize, m.a(MADE) as usize);
        for k in 0..16 {
            m.put(0x5EA0CC + 4 * k, g(MADE + 4 * k));
        }
        let made = m.a(MADE);
        for (first, out, negate) in [
            (0, 0x5C568C, true),
            (0, 0x5C5698, false),
            (3, 0x5C56A4, true),
            (3, 0x5C56B0, false),
        ] {
            let p = [
                int_80(f[first], negate),
                int_80(f[first + 1], false),
                int_80(f[first + 2], false),
            ];
            let r = rotate_point(made, p);
            for (k, e) in r.iter().enumerate() {
                m.put(out + 4 * k as u32, ext_to_int(*e) as u32);
            }
        }
        let on = g(0x6415E0) != 0;
        let (a, b) = (g(0x70FCEC), g(0x70FCF0));
        m.put(0x5C56BC, if on { a << 2 } else { 0 });
        m.put(0x5C56E8, a.wrapping_add(a));
        m.put(0x5C56C0, a.wrapping_add(a));
        m.put(0x5C56C4, if on { b >> 3 } else { 0 });
        m.put(0x5C56E4, b << 3);
        m.put(0x5C56D0, if (a != 0 || b != 0) && on { 0 } else { 1 });
        m.put(0x5C56C8, 1);
        m.put(0x5C56CC, 0);
        let old = g(0x70F628);
        if old != 0 {
            m.put(0x5C5680, old);
        }
        m.put(0x70F628, 0);
        m.put(0x5C5684, u32::from(g(0x70FA2C) != 0));
        m.put(0x62400C, 0x2000_0000);
        m.put(
            0x5C5660,
            int_over_int_plus(g(0x70FA3C) as i32, 0x2000_0000, g(0x637644)),
        );
        m.put(0x5C5624, x87::loaded(g(0x70F5C4)));
        m.put(0x5C5628, x87::loaded(g(0x7102F4)));
        if g(0x601024) != 0 {
            m.put(0x5C5684, 0);
            m.put(0x5C5680, 0);
        }
        Ok(Left::nothing()
            .with(EAX, old)
            .with(EBX, 0x10000)
            .with(ECX, 0)
            .with(EDX, 0)
            .with(ESI, made)
            .with(EDI, m.a(0x5EA10C))
            .with(EBP, made))
    }
}

/// The Rust version.
pub unsafe fn set_listener() -> Result<Left, Refused> {
    unsafe { set_listener_at(&Mirror::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { set_listener() }
}

/// What it writes.
pub fn regions(m: &Mirror) -> Vec<Region> {
    vec![
        (m.a(0x5C2FA0), 0x38),
        (m.a(0x5C5620), 0xCC),
        (m.a(0x5EA0CC), 0x40),
        (m.a(TURNED), 0x80),
        (m.a(MADE), 0x40),
        (m.a(0x608C60), 0x18),
        (m.a(0x62400C), 4),
        (m.a(0x70F628), 4),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(&Mirror::game()), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let m = Mirror::of(&RANGES);
    let copy = match m.copy(SPEC, &|_| None) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    // The viewed car's block is the test's (the game's may not be there).
    let mut block = vec![0u32; 0xFE0 / 4];
    let b = block.as_mut_ptr() as u32;
    let mut all = m.regions();
    all.push((b, 0xFE0));
    let fill = |seed: u32| {
        let mut x = seed.wrapping_mul(0x9E37_79B9) | 1;
        let mut next = move || {
            x = x.wrapping_mul(1_103_515_245).wrapping_add(12345);
            x >> 4
        };
        unsafe {
            m.put(0x63DB9C, next() % 8);
            for k in 0..8 {
                m.put(0x71B3B8 + 4 * k, b);
            }
            for k in 0..(0xFE0 / 4) {
                (b as *mut u32).add(k).write(next());
            }
            (b as *mut u32).add(0xABC / 4).write(next() % 0x20);
            m.put(0x70F410, next() % 0x22);
            for v in [0x6238A8, 0x6415E0, 0x70FA2C, 0x601024] {
                m.put(v, next() % 2);
            }
            for v in [0x70FCEC, 0x70FCF0, 0x70F628] {
                m.put(v, if next() % 3 == 0 { 0 } else { next() % 0x400 });
            }
            m.put(0x70FA3C, next());
            for k in 0..3 {
                m.put(0x606298 + 4 * k, next());
            }
            for k in 0..32 {
                m.put(
                    0x70FC2C + 4 * k,
                    ((next() % 2000) as f32 / 1000.0 - 1.0).to_bits(),
                );
            }
        }
    };
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 1..=8u32 {
                let saved = capture(&all);
                fill(seed);
                let input = capture(&all);
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = capture(&all);
                restore(&all, &input);
                let mine = unsafe { set_listener_at(&m) };
                let state = capture(&all);
                let regs_ok = mine.as_ref().is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
                t.check(regs_ok && theirs == state, || {
                    let first = theirs.iter().zip(&state).position(|(a, b)| a != b);
                    format!(
                        "control word {word:#x}, seed {seed}: regs_ok={regs_ok} {mine:?}, first difference at byte {first:?}"
                    )
                });
                restore(&all, &saved);
            }
        });
    }
    let saved = capture(&all);
    unsafe { m.put(0x63DB9C, 8) };
    let before = capture(&all);
    let refused = unsafe { set_listener_at(&m) }.is_err();
    t.check(refused && capture(&all) == before, || {
        "viewed car 8: not refused, or something written".into()
    });
    restore(&all, &saved);
    drop((copy, block));
    t.finish()
}
