//! `physics::engine_torque` (0x43CBBB): the engine's torque from its speed. s = engine
//! speed / its top (+0x1DC) into `dword_62400C`; for s in 0 ..= below 1 (`flt_5C5D90`,
//! `flt_5C5D38`; unordered counts as out) the curve's entry i = s x 8 (`fistp` with the
//! rounding set to chop) into `dword_623FB8` (ebx), and `flt_614748` the line between
//! (`flt_613FF0[i]`, `dword_614B7C[i]`) and (`flt_613FF0[i + 1]`, `dword_614B7C[i + 1]`)
//! at s; else `dword_614B7C[0]`. Then `flt_614744` = engine / top x `flt_613F94` (0.25),
//! and `flt_614748` times +0x3F4 (`unk_71A3CC`, as a float).
//!
//! In:  nothing
//! Out: ebx i on the curve's path; the rest kept; no flags read
//!
//! Callers: `physics::engine_torque_or_cut`, `sub_421848`, `sub_43CAAB`, `sub_43EA80`.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back), against the converted version: engine speeds across the curve and past it,
//! exact at double precision (`test_engine_floats`).

use crate::errors::Refused;
use crate::framework::{
    EBX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::math::x87;
use crate::math::x87_run::{
    TEST_CONTROL_WORDS, div_then_mul, lerp_table, trunc_mul_to_int, with_control_word,
};
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::floats::{
    FLOAT, Floats, at_test_precisions, check_state_floats, dispatch_state_floats,
};
use crate::service::harness::{Region, capture, check_state, restore};
use rc2k_core::physics::engine::{Engine, engine_torque as engine_torque_core};
use std::cmp::Ordering;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::engine_torque", rva(0x43CBBB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

pub fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// C0 of comparing `a` with `b`: below, or unordered.
pub fn below(a: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The converted version (`rc2k_core::physics::engine`, plain floats): the hook's; ebx
/// when the curve was read.
// Converted: `rc2k_core::physics::engine::engine_torque`.
pub unsafe fn engine_torque_plain() -> Left {
    let f = |x: usize| f32::from_bits(g(x));
    let engine = Engine {
        speed: f(0x71A1B0),
        top: f(0x71A1B4),
        zero: f(0x5C5D90),
        one: f(0x5C5D38),
        steps: f(0x5C5D50),
        quarter: f(0x613F94),
        factor: f(0x71A3CC),
    };
    let curve = |i: u32| {
        let k = 4 * i as usize;
        (
            [f(0x613FF0 + k), f(0x613FF4 + k)],
            [f(0x614B7C + k), f(0x614B80 + k)],
        )
    };
    let t = engine_torque_core(&engine, curve, || f(0x614B7C));
    p(0x62400C, t.share.to_bits());
    let mut left = Left::nothing();
    if let Some(i) = t.entry {
        p(0x623FB8, i);
        left = left.with(EBX, i);
    }
    p(0x614744, t.revs.to_bits());
    p(0x614748, t.torque.to_bits());
    left
}

/// The Rust version (the exact one, for `physics::engine_torque_or_cut`, `drive_force`,
/// `free_rev*`, `test_drive_run`): ebx when the curve was read.
pub unsafe fn engine_torque() -> Left {
    let mut left = Left::nothing();
    let s = x87::div(g(0x71A1B0), g(0x71A1B4));
    p(0x62400C, s);
    if below(s, g(0x5C5D38)) && !below(s, g(0x5C5D90)) {
        let i = trunc_mul_to_int(s, g(0x5C5D50)) as u32;
        p(0x623FB8, i);
        left = left.with(EBX, i);
        let k = 4 * i as usize;
        p(
            0x614748,
            lerp_table(
                s,
                g(0x613FF0 + k),
                g(0x613FF4 + k),
                g(0x614B7C + k),
                g(0x614B80 + k),
            ),
        );
    } else {
        p(0x614748, x87::loaded(g(0x614B7C)));
    }
    p(
        0x614744,
        div_then_mul(g(0x71A1B0), g(0x71A1B4), g(0x613F94)),
    );
    p(0x614748, x87::mul(g(0x614748), g(0x71A3CC)));
    left
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { engine_torque_plain() })
}

/// The float places `engine_torque` writes (IDA addresses, lengths).
pub const TORQUE_FLOATS: [(usize, usize); 2] = [(0x62400C, 4), (0x614744, 8)];

/// What the engine's steps write.
pub fn regions() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![
        (a(0x62400C), 4),
        (a(0x623FB8), 4),
        (a(0x614740), 0x14),
        (a(0x71A1C8), 4),
    ]
}

/// The shadow comparison of a converted engine step: `regions`, the float places
/// `floats` names (IDA addresses, lengths) within `FLOAT`.
pub fn dispatch_engine(
    hook: &'static RawHook,
    r: &mut Regs,
    call_original: crate::screens::harness::CallOriginal,
    regions: &[Region],
    floats: &[(usize, usize)],
    rust: &dyn Fn(&Regs) -> Result<Left, Refused>,
) {
    let memory: Vec<_> = floats
        .iter()
        .map(|&(x, len)| ((game_addr(rva(x)) as u32, len), FLOAT))
        .collect();
    let floats = Floats {
        memory: &memory,
        regs: &[],
    };
    dispatch_state_floats(hook, r, call_original, regions, &floats, rust);
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_engine(&HOOK, r, call_original, &regions(), &TORQUE_FLOATS, &rust);
}

/// The inputs the engine tests set.
pub fn inputs() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![
        (a(0x71A1B0), 8),
        (a(0x613FF0), 0x24),
        (a(0x614B7C), 0x24),
        (a(0x71A3CC), 4),
        (a(0x71A610), 4),
        (a(0x71A1AC), 4),
        (a(0x6140BC), 0x24),
        (a(0x5CB968), 4),
        (a(0x71A1A8), 4),
        (a(0x71A018), 4),
        (a(0x71A184), 4),
        (a(0x71A000), 4),
        (a(0x614758), 4),
        (a(0x614750), 4),
        (a(0x624020), 4),
        (a(0x62401C), 4),
    ]
}

/// An engine case: the engine speed's share of its top, the curve and the rest by `seed`.
pub fn set_engine(seed: u32, share: f32) {
    p(0x71A1B4, 6000.0f32.to_bits());
    p(0x71A1B0, (6000.0 * share).to_bits());
    let mut f = [0u32; 9];
    test_floats(seed * 7 + 2, &mut f);
    for (k, v) in f.iter().enumerate() {
        p(0x613FF0 + 4 * k, (k as f32 / 8.0).to_bits());
        p(0x614B7C + 4 * k, (f32::from_bits(*v) % 400.0).to_bits());
    }
    p(0x71A3CC, (0.5 + seed as f32 * 0.25).to_bits());
    p(0x71A610, 3 + seed % 4);
    p(0x71A1AC, seed % 5);
    for (k, r) in [-4.0f32, 0.0, 3.5, 2.2, 1.5, 1.1, 0.9, 0.8, 0.7]
        .iter()
        .enumerate()
    {
        p(0x6140BC + 4 * k, r.to_bits());
    }
    p(0x5CB968, 1.5f32.to_bits());
    p(0x71A1A8, seed.is_multiple_of(3) as u32);
    p(0x71A018, seed % 2);
    p(0x71A184, (seed as f32 * 4.5).to_bits());
    p(0x71A000, seed * 3);
    p(0x71A1C8, seed % 3);
    p(0x614750, 0x5A5A);
}

/// The engine shares the tests run through.
pub const SHARES: [f32; 8] = [0.0, 0.1, 0.33, 0.5, 0.875, 0.99, 1.0, 1.2];

/// The test of one of the engine's steps.
pub fn test_engine(
    t: &mut SelfTest,
    original: unsafe extern "C" fn(*mut Regs),
    rust: &dyn Fn(&Regs) -> Result<Left, Refused>,
) {
    let mut all = regions();
    all.extend(inputs());
    let saved = capture(&all);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for (seed, share) in SHARES.iter().enumerate() {
                check_state(
                    t,
                    &format!("control word {word:#x}, share {share}"),
                    original,
                    Regs::SENTINEL,
                    &|| set_engine(seed as u32, *share),
                    &all,
                    rust,
                );
            }
        });
    }
    restore(&all, &saved);
}

/// `test_engine` for a converted step: exact at double precision, at extended the float
/// places `floats` names (IDA addresses, lengths) within `FLOAT`.
pub fn test_engine_floats(
    t: &mut SelfTest,
    name: &'static str,
    original: unsafe extern "C" fn(*mut Regs),
    rust: &dyn Fn(&Regs) -> Result<Left, Refused>,
    floats: &[(usize, usize)],
) {
    let mut all = regions();
    all.extend(inputs());
    let saved = capture(&all);
    let memory: Vec<_> = floats
        .iter()
        .map(|&(x, len)| ((game_addr(rva(x)) as u32, len), FLOAT))
        .collect();
    let tolerant = Floats {
        memory: &memory,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, fl| {
        for (seed, share) in SHARES.iter().enumerate() {
            check_state_floats(
                t,
                name,
                &format!("control word {word:#x}, share {share}"),
                original,
                Regs::SENTINEL,
                &|| set_engine(seed as u32, *share),
                &all,
                fl,
                rust,
            );
        }
    });
    restore(&all, &saved);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_engine_floats(
        &mut t,
        "physics::engine_torque",
        call_original,
        &rust,
        &TORQUE_FLOATS,
    );
    t.finish()
}
