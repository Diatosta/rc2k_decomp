//! `physics::wheel_springs` (0x43ECD4): the four wheels' springs from their heights. For
//! each wheel k (ebx 0 .. 3):
//! - the last squeeze `qword_71A5D8[k]` (g_car +0x600) kept in `dword_623FB8` and
//!   `flt_71A5E8[k]` (+0x610);
//! - the new squeeze = -`dword_6D2514[k]` (the wheel's height, through `dword_62400C`,
//!   `fild`) over 65536 (`flt_5C5D98`) and `flt_614BF8`, held to 0 ..= 1
//!   (`flt_5C5D90`, `flt_5C5D38`);
//! - its force `flt_614178[k]` = -(squeeze x `flt_613F90` x `flt_614774`); the damping
//!   `flt_614124[k]` = (squeeze - last) x `flt_614C00`, times `flt_614BEC` when below 0
//!   (or unordered), added to the force unless the squeeze is 0 (or unordered).
//!
//! Then `dword_71A8E8[k]` = `fistp` of ((`flt_70FFF4` - `flt_614BF4` + `flt_613EC0`) +
//! (squeeze - `flt_613F8C`) x `flt_614BF8`) x 65536; the forces into the wheel records'
//! +4 (`unk_614830` + 0x20 k); the records' +0xC .. +0x14 from the wheels' places
//! (`flt_71AAC8` .., `flt_71ABC8` .., `flt_71ACC8` ..), copied on into the records at
//! 0x6148DC .. and 0x61495C ..; `flt_6148C0` 0; minus `flt_6141D4` into 0x6148E0,
//! 0x614900, 0x614920, 0x614940 and minus `flt_6141D8` into 0x614960 .. 0x6149C0.
//!
//! In:  nothing
//! Out: ebx 4, eax minus wheel 3's height; the rest kept; no flags read (the next is a
//!      call)
//!
//! Caller: `sub_43B635` (0x43B813), every physics step. Its loop goes back to +5, just
//! past the patched bytes (`mov ebx, 0` is five): no private copy needed.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back), against the converted version: heights by lot across the spring's range, last
//! squeezes, the factors, exact at double precision.

use crate::errors::Refused;
use crate::framework::{
    EAX, EBX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::math::x87;
use crate::math::x87_run::{ext_div2, int_80, neg_mul2, neg_rounded, sub_mul, wheel_int};
use crate::scene::matrix3::{extended, test_floats};
use crate::screens::harness::Left;
use crate::service::floats::{
    FLOAT, Floats, ROUNDED, Tolerance, at_test_precisions, check_state_floats,
    dispatch_state_floats,
};
use crate::service::harness::{Region, capture, restore};
use rc2k_core::physics::springs::{SpringFactors, spring, wheel_place};
use rc2k_core::physics::x87::{loaded, negated};
use std::cmp::Ordering;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::wheel_springs", rva(0x43ECD4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn a(x: usize) -> u32 {
    game_addr(rva(x)) as u32
}

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

const SQUEEZE: usize = 0x71A5D8; // g_car + 0x600, four floats
const LAST: usize = 0x71A5E8; // + 0x610
const HEIGHTS: usize = 0x6D2514; // the wheels' heights (ints)
const FORCE: usize = 0x614178;
const DAMPING: usize = 0x614124;
const ZERO: usize = 0x5C5D90;
const ONE: usize = 0x5C5D38;

/// `fld b; fld x; fcompp` (or `fld x; fcomp b`), C0: x below b, or unordered.
fn below(x: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(x).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn wheel_springs() -> Left {
    let mut eax = 0;
    for k in 0..4 {
        let q = SQUEEZE + 4 * k;
        let old = x87::loaded(g(q));
        p(0x623FB8, old);
        p(LAST + 4 * k, old);
        eax = g(HEIGHTS + 4 * k).wrapping_neg();
        p(0x62400C, eax);
        p(
            q,
            ext_div2(int_80(eax as i32, false), g(0x5C5D98), g(0x614BF8)),
        );
        if below(g(q), g(ZERO)) {
            p(q, x87::loaded(g(ZERO)));
        }
        if !below(g(q), g(ONE)) {
            p(q, x87::loaded(g(ONE)));
        }
        p(FORCE + 4 * k, neg_mul2(g(q), g(0x613F90), g(0x614774)));
        p(DAMPING + 4 * k, sub_mul(g(q), g(0x623FB8), g(0x614C00)));
        if below(g(DAMPING + 4 * k), g(ZERO)) {
            p(DAMPING + 4 * k, x87::mul(g(DAMPING + 4 * k), g(0x614BEC)));
        }
        let zero_or_none = f32::from_bits(g(q))
            .partial_cmp(&f32::from_bits(g(ZERO)))
            .is_none_or(|o| o == Ordering::Equal);
        if !zero_or_none {
            p(
                FORCE + 4 * k,
                x87::add(g(DAMPING + 4 * k), g(FORCE + 4 * k)),
            );
        }
    }
    for k in 0..4 {
        let v = wheel_int([
            g(0x70FFF4),
            g(0x614BF4),
            g(0x613EC0),
            g(SQUEEZE + 4 * k),
            g(0x613F8C),
            g(0x614BF8),
            g(0x5C5D98),
        ]);
        p(0x71A8E8 + 4 * k, v as u32);
    }
    for k in 0..4 {
        p(0x614834 + 0x20 * k, x87::loaded(g(FORCE + 4 * k)));
    }
    for (row, from) in [0x71AAC8usize, 0x71ABC8, 0x71ACC8].into_iter().enumerate() {
        for k in 0..4 {
            p(0x61483C + 4 * row + 0x20 * k, x87::loaded(g(from + 4 * k)));
        }
    }
    for (from, to) in RECORD_COPIES {
        let v = x87::loaded(g(from));
        for t in to {
            p(t, v);
        }
    }
    p(0x6148C0, 0);
    let n = neg_rounded(extended(g(0x6141D4)));
    for t in [0x6148E0, 0x614900, 0x614920, 0x614940] {
        p(t, n);
    }
    let n = neg_rounded(extended(g(0x6141D8)));
    for t in [0x614960, 0x614980, 0x6149A0, 0x6149C0] {
        p(t, n);
    }
    Left::nothing().with(EAX, eax).with(EBX, 4)
}

/// The converted version (`rc2k_core::physics::springs`, plain floats): the hook's.
// Converted: `rc2k_core::physics::springs::{spring, wheel_place}`.
pub unsafe fn wheel_springs_plain() -> Left {
    let f = |x: usize| f32::from_bits(g(x));
    let factors = SpringFactors {
        scale: f(0x5C5D98),
        range: f(0x614BF8),
        zero: f(ZERO),
        one: f(ONE),
        stiffness: f(0x613F90),
        load: f(0x614774),
        damping: f(0x614C00),
        rebound: f(0x614BEC),
    };
    let mut eax = 0;
    for k in 0..4 {
        let q = SQUEEZE + 4 * k;
        let last = f(q);
        let old = loaded(last).to_bits();
        p(0x623FB8, old);
        p(LAST + 4 * k, old);
        let height = g(HEIGHTS + 4 * k);
        eax = height.wrapping_neg();
        p(0x62400C, eax);
        let s = spring(height, last, &factors);
        p(q, s.squeeze.to_bits());
        p(FORCE + 4 * k, s.force.to_bits());
        p(DAMPING + 4 * k, s.damping.to_bits());
    }
    let abcd = [f(0x70FFF4), f(0x614BF4), f(0x613EC0), f(0x613F8C)];
    for k in 0..4 {
        let v = wheel_place(abcd, f(SQUEEZE + 4 * k), factors.range, factors.scale);
        p(0x71A8E8 + 4 * k, v as u32);
    }
    // The wheel records: the forces and the wheels' places, copied on (`fld; fstp`).
    let copy = |to: usize, from: usize| p(to, loaded(f(from)).to_bits());
    for k in 0..4 {
        copy(0x614834 + 0x20 * k, FORCE + 4 * k);
    }
    for (row, from) in [0x71AAC8usize, 0x71ABC8, 0x71ACC8].into_iter().enumerate() {
        for k in 0..4 {
            copy(0x61483C + 4 * row + 0x20 * k, from + 4 * k);
        }
    }
    for (from, to) in RECORD_COPIES {
        for t in to {
            copy(t, from);
        }
    }
    p(0x6148C0, 0);
    let n = negated(f(0x6141D4)).to_bits();
    for t in [0x6148E0, 0x614900, 0x614920, 0x614940] {
        p(t, n);
    }
    let n = negated(f(0x6141D8)).to_bits();
    for t in [0x614960, 0x614980, 0x6149A0, 0x6149C0] {
        p(t, n);
    }
    Left::nothing().with(EAX, eax).with(EBX, 4)
}

/// The records' places copied on into the records at 0x6148DC .. and 0x61495C ...
const RECORD_COPIES: [(usize, [usize; 2]); 8] = [
    (0x61483C, [0x6148DC, 0x61495C]),
    (0x614844, [0x6148E4, 0x614964]),
    (0x61485C, [0x6148FC, 0x61497C]),
    (0x614864, [0x614904, 0x614984]),
    (0x61487C, [0x61491C, 0x61499C]),
    (0x614884, [0x614924, 0x6149A4]),
    (0x61489C, [0x61493C, 0x6149BC]),
    (0x6148A4, [0x614944, 0x6149C4]),
];

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { wheel_springs_plain() })
}

/// The float places and what they are held to at extended precision.
fn floats() -> Vec<(Region, Tolerance)> {
    vec![
        ((a(SQUEEZE), 0x20), FLOAT),
        ((a(DAMPING), 0x10), FLOAT),
        ((a(FORCE), 0x10), FLOAT),
        ((a(0x71A8E8), 0x10), ROUNDED),
        ((a(0x614830), 0x1A0), FLOAT),
    ]
}

fn regions() -> Vec<Region> {
    vec![
        (a(SQUEEZE), 0x20),
        (a(0x623FB8), 4),
        (a(0x62400C), 4),
        (a(DAMPING), 0x10),
        (a(FORCE), 0x10),
        (a(0x71A8E8), 0x10),
        (a(0x614830), 0x1A0),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    let memory = floats();
    let floats = Floats {
        memory: &memory,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The factors the test sets (each one float).
    const FACTORS: [usize; 9] = [
        0x614BF8, 0x613F90, 0x614774, 0x614C00, 0x614BEC, 0x70FFF4, 0x614BF4, 0x613EC0, 0x613F8C,
    ];
    let mut all = regions();
    all.push((a(HEIGHTS), 0x10));
    all.extend(FACTORS.iter().map(|&f| (a(f), 4)));
    all.extend([
        (a(0x71AAC8), 0x10),
        (a(0x71ABC8), 0x10),
        (a(0x71ACC8), 0x10),
    ]);
    all.push((a(0x6141D4), 8));
    let saved = capture(&all);
    let memory = floats();
    let tolerant = Floats {
        memory: &memory,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, fl| {
        {
            for seed in 1..=10u32 {
                let set = || {
                    let mut f = [0u32; 32];
                    test_floats(seed * 23 + 1, &mut f);
                    // Heights about the spring's range: -0x18000 .. 0x8000 (with the
                    // range factor 1, squeezes -0.5 .. 1.5).
                    for k in 0..4u32 {
                        let h = (seed.wrapping_mul(0x9E37) ^ k.wrapping_mul(0x3C6E)) % 0x20000;
                        p(HEIGHTS + 4 * k as usize, h.wrapping_sub(0x18000));
                    }
                    p(0x614BF8, if seed == 3 { 0 } else { 1.0f32.to_bits() });
                    for (n, &fa) in FACTORS.iter().enumerate().skip(1) {
                        p(fa, f[n]);
                    }
                    for (n, &v) in f.iter().enumerate().skip(9).take(4) {
                        p(SQUEEZE + 4 * (n - 9), v);
                    }
                    for (n, &v) in f.iter().enumerate().skip(13).take(12) {
                        let row = (n - 13) / 4;
                        p([0x71AAC8, 0x71ABC8, 0x71ACC8][row] + 4 * ((n - 13) % 4), v);
                    }
                    p(0x6141D4, f[25]);
                    p(0x6141D8, f[26]);
                };
                check_state_floats(
                    &mut t,
                    "physics::wheel_springs",
                    &format!("control word {word:#x}, seed {seed}"),
                    call_original,
                    Regs::SENTINEL,
                    &set,
                    &all,
                    fl,
                    &rust,
                );
            }
        }
    });
    restore(&all, &saved);
    t.finish()
}
