//! `physics::set_wheel_weights` (0x43F3E7): the four wheels' load forces (the records at
//! 0x614770 + 0x20 k, `physics::clear_forces`'): a share `flt_6D3070` held to 1 ..
//! `flt_5C5D48` (through `dword_62400C`), ((it - 1) `flt_5C5DA4` + 1) gravity
//! (`flt_613FC8`) `flt_614170`, negated, times `flt_5C5DA4` again into each record's +4,
//! turned into the car's frame (`physics::turn_by_inverse`); then each record's second
//! vector (+0xC ..) set to (-1 / 1, 0, ±1) by its corner and turned the same way. A
//! reading: the weight on each wheel, the corners' places.
//!
//! In:  nothing
//! Out: esi 0x6147DC (the last turned); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B723), every physics step.
//! Self-test: a copy of the original on test records (its eight calls to
//! `turn_by_inverse` kept), shares below, in and above the range under three precisions.

use crate::errors::Refused;
use crate::framework::{
    ESI, RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva,
};
use crate::math::x87;
use crate::math::x87_run::{TEST_CONTROL_WORDS, compare_full, weight_force, with_control_word};
use crate::physics::turn_by_inverse::turn_by_inverse;
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::set_wheel_weights",
    rva(0x43F3E7),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The share, the scratch, the weight, the records (0x80 bytes).
pub struct Places {
    pub share: u32,
    pub scratch: u32,
    pub weight: u32,
    pub records: u32,
}

impl Places {
    fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            share: a(0x6D3070),
            scratch: a(0x62400C),
            weight: a(0x614170),
            records: a(0x614770),
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust core on the variables at `p`; `r` as the original leaves it.
pub unsafe fn weights_at(p: &Places, r: &mut Regs) {
    unsafe {
        let one: u32 = game_read(rva(0x5C5D38));
        let high: u32 = game_read(rva(0x5C5D48));
        let k: u32 = game_read(rva(0x5C5DA4));
        put(p.scratch, x87::loaded(at(p.share)));
        if compare_full(at(p.scratch), one) & 0x100 != 0 {
            put(p.scratch, x87::loaded(one));
        }
        if compare_full(at(p.scratch), high) & 0x100 == 0 {
            put(p.scratch, x87::loaded(high));
        }
        let f = weight_force(
            at(p.scratch),
            one,
            k,
            game_read(rva(0x613FC8)),
            at(p.weight),
        );
        for c in 0..4 {
            put(p.records + 0x20 * c + 4, f);
        }
        for c in 0..4 {
            turn_by_inverse(p.records + 0x20 * c);
        }
        let plus = x87::loaded(1.0f32.to_bits());
        let minus = x87::negated(plus);
        for o in [0x2C, 0x6C, 0x14, 0x34] {
            put(p.records + o, plus);
        }
        for o in [0x0C, 0x4C, 0x54, 0x74] {
            put(p.records + o, minus);
        }
        for c in 0..4 {
            r.esi = p.records + 0x20 * c + 0xC;
            turn_by_inverse(r.esi);
        }
    }
}

/// The Rust version; `r` as the original leaves it.
pub unsafe fn set_wheel_weights(r: &mut Regs) {
    unsafe { weights_at(&Places::game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { set_wheel_weights(&mut g) };
    Ok(Left::nothing().with(ESI, g.esi))
}

extern "C" fn dispatch(r: &mut Regs) {
    let p = Places::game();
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(p.scratch, 4), (p.records, 0x80)],
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 3 + 0x20];
    let base = cells.as_mut_ptr() as u32;
    let p = Places {
        share: base,
        scratch: base + 4,
        weight: base + 8,
        records: base + 12,
    };
    let mut variables = vec![
        (0x6D3070usize, p.share),
        (0x62400C, p.scratch),
        (0x614170, p.weight),
    ];
    for k in 0..0x20u32 {
        variables.push((0x614770 + 4 * k as usize, p.records + 4 * k));
    }
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43F3E7,
            len: 0x121,
            entry: &[0xD9, 0x05, 0x70, 0x30, 0x6D, 0x00],
            calls: &[0x9A, 0xA5, 0xB0, 0xBB, 0xFA, 0x105, 0x110, 0x11B],
            jumps_out: &[],
        },
        &|_| None,
        &variables,
        28,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 4 * cells.len())];
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for (n, share) in [0.5f32, 1.0, 1.3, 2.0, 9.0, f32::NAN]
                .into_iter()
                .enumerate()
            {
                let fill = || unsafe {
                    for k in 0..(3 + 0x20) {
                        put(base + 4 * k, ((k as f32 - 10.0) * 0.75).to_bits());
                    }
                    put(p.share, share.to_bits());
                    put(p.weight, (1200.0 + n as f32 * 100.0).to_bits());
                };
                fill();
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = capture(&all);
                fill();
                let mut g = Regs::SENTINEL;
                unsafe { weights_at(&p, &mut g) };
                let mine = capture(&all);
                let regs_ok = Left::nothing()
                    .with(ESI, g.esi)
                    .matches(&Regs::SENTINEL, &ro);
                t.check(regs_ok && theirs == mine, || {
                    let first = theirs.iter().zip(&mine).position(|(a, b)| a != b);
                    format!("control word {word:#x}, share {share}: regs_ok={regs_ok}, first difference at byte {first:?}")
                });
            }
        });
    }
    drop((copy, cells));
    t.finish()
}
