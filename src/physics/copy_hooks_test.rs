//! For the small physics hooks that only move dwords between fixed variables: a self-test
//! of a copy of the original whose variables are a test block's, against the Rust core
//! on the same block, for several fillings; `test_moves_floats` for the converted float
//! cores, within the float pass's tolerances.

use crate::framework::{GameHook, RawHook, Regs, SelfTest, call_game};
use crate::screens::harness::Left;
use crate::service::floats::{
    Agreement, Floats, Tolerance, at_test_precisions, compare_memory, compare_regs, note_drift,
};
use crate::service::harness::{Region, capture};

/// `vars` (IDA addresses) become `block[k]`; `core(addresses, regs)` the Rust core;
/// `fill(seed, cells)` a case's inputs; `references` the copy's count; `entry` its first
/// bytes; `len` the original's length.
#[allow(clippy::too_many_arguments)] // one test's whole description
pub fn test_moves(
    hook: &RawHook,
    address: usize,
    len: usize,
    entry: &[u8],
    vars: &[usize],
    references: usize,
    cases: u32,
    fill: &dyn Fn(u32, &mut [u32]),
    core: &dyn Fn(&[u32], &mut Regs) -> Left,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; vars.len()];
    let base = cells.as_mut_ptr() as u32;
    let at: Vec<u32> = (0..vars.len() as u32).map(|k| base + 4 * k).collect();
    let variables: Vec<(usize, u32)> = vars.iter().copied().zip(at.iter().copied()).collect();
    let copy = match crate::audio::test_copy::make(
        hook,
        &crate::audio::test_copy::Copy {
            address,
            len,
            entry,
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &variables,
        references,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let region = [(base, 4 * vars.len())];
    let set = |seed: u32| {
        let mut v = vec![0u32; vars.len()];
        fill(seed, &mut v);
        for (k, x) in v.into_iter().enumerate() {
            unsafe { (at[k] as *mut u32).write_volatile(x) };
        }
    };
    for seed in 0..cases {
        set(seed);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&region);
        set(seed);
        let mut g = Regs::SENTINEL;
        let left = core(&at, &mut g);
        let mine = capture(&region);
        let regs_ok = left.matches(&Regs::SENTINEL, &ro);
        t.check(regs_ok && theirs == mine, || {
            format!("case {seed}: regs_ok={regs_ok} original {theirs:x?}; rust {mine:x?}")
        });
    }
    drop((copy, cells));
    t.finish()
}

/// Dwords by lot for a case.
pub fn lot(seed: u32, out: &mut [u32]) {
    let mut s = seed.wrapping_mul(0x9E37_79B9) | 1;
    for v in out.iter_mut() {
        s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
        *v = s;
    }
}

/// Two agreements together: a difference anywhere differs, drifts add up.
pub fn both(a: Agreement, b: Agreement) -> Agreement {
    match (a, b) {
        (Agreement::Differ, _) | (_, Agreement::Differ) => Agreement::Differ,
        (Agreement::Drift(x), Agreement::Drift(y)) => Agreement::Drift(x + y),
        (Agreement::Drift(x), _) | (_, Agreement::Drift(x)) => Agreement::Drift(x),
        _ => Agreement::Same,
    }
}

/// Counts one float case: a drift passes (and is noted), a difference fails.
pub fn check_agreement(
    t: &mut SelfTest,
    name: &'static str,
    agreement: Agreement,
    detail: impl FnOnce() -> String,
) {
    if let Agreement::Drift(n) = agreement {
        note_drift(name, || format!("self-test: {n} places"));
    }
    t.check(agreement != Agreement::Differ, detail);
}

/// `test_moves` for a converted float core (the float pass): at the tests' control words,
/// exact at double precision, at extended the cells `floats` names (index, tolerance) and
/// the registers `regs` within their tolerances.
#[allow(clippy::too_many_arguments)] // one test's whole description
pub fn test_moves_floats(
    hook: &RawHook,
    address: usize,
    len: usize,
    entry: &[u8],
    vars: &[usize],
    references: usize,
    cases: u32,
    fill: &dyn Fn(u32, &mut [u32]),
    core: &dyn Fn(&[u32], &mut Regs) -> Left,
    floats: &[(usize, Tolerance)],
    regs: &[(u8, Tolerance)],
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; vars.len()];
    let base = cells.as_mut_ptr() as u32;
    let at: Vec<u32> = (0..vars.len() as u32).map(|k| base + 4 * k).collect();
    let variables: Vec<(usize, u32)> = vars.iter().copied().zip(at.iter().copied()).collect();
    let copy = match crate::audio::test_copy::make(
        hook,
        &crate::audio::test_copy::Copy {
            address,
            len,
            entry,
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &variables,
        references,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let region = [(base, 4 * vars.len())];
    let set = |seed: u32| {
        let mut v = vec![0u32; vars.len()];
        fill(seed, &mut v);
        for (k, x) in v.into_iter().enumerate() {
            unsafe { (at[k] as *mut u32).write_volatile(x) };
        }
    };
    let memory: Vec<(Region, Tolerance)> = floats.iter().map(|&(k, tl)| ((at[k], 4), tl)).collect();
    let tolerant = Floats {
        memory: &memory,
        regs,
    };
    let name = hook.name();
    at_test_precisions(&tolerant, &mut |word, fl| {
        for seed in 0..cases {
            set(seed);
            let mut ro = Regs::SENTINEL;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = capture(&region);
            set(seed);
            let mut g = Regs::SENTINEL;
            let left = core(&at, &mut g);
            let mine = capture(&region);
            let agreement = both(
                compare_memory(&region, &theirs, &mine, fl),
                compare_regs(&left, &Regs::SENTINEL, &ro, fl),
            );
            check_agreement(&mut t, name, agreement, || {
                format!(
                    "control word {word:#x}, case {seed}: original {theirs:x?} eax={:#x} ebx={:#x} ecx={:#x} edx={:#x}; rust {mine:x?} {left:x?}",
                    ro.eax, ro.ebx, ro.ecx, ro.edx
                )
            });
        }
    });
    drop((copy, cells));
    t.finish()
}

/// A copy test's float case at control word `word`: the captures of `regions` the same at
/// double precision; at the others the places `floats` names within their tolerances (a
/// drift noted). `regs_ok`: the registers as the original left them.
#[allow(clippy::too_many_arguments)] // one case's whole description
pub fn check_floats_at(
    t: &mut SelfTest,
    name: &'static str,
    word: u16,
    regions: &[Region],
    theirs: &[u8],
    mine: &[u8],
    floats: &[(Region, Tolerance)],
    regs_ok: bool,
    detail: impl FnOnce() -> String,
) {
    let tolerant = Floats {
        memory: floats,
        regs: &[],
    };
    let fl = if word == crate::math::x87_run::DOUBLE {
        &Floats::EXACT
    } else {
        &tolerant
    };
    let agreement = if regs_ok {
        compare_memory(regions, theirs, mine, fl)
    } else {
        Agreement::Differ
    };
    check_agreement(t, name, agreement, detail);
}
