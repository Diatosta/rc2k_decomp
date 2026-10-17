//! For the small physics hooks that only move dwords between fixed variables: a self-test
//! of a copy of the original whose variables are a test block's, against the Rust core
//! on the same block, for several fillings.

use crate::framework::{RawHook, Regs, SelfTest, call_game};
use crate::screens::harness::Left;
use crate::service::harness::capture;

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
