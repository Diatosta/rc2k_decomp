//! `physics::stall_when_still` (0x440B9D): the length of the motion kept by
//! `physics::keep_motion` (0x61409C .., `physics::vector_length`, rounded into
//! `dword_62400C`); when not 0 (C3 clear), `flt_6140AC` = the motion (0x71A17C ..) dot it
//! (`physics::dot_product`) over the length twice. Then, unless the length's size is
//! below `flt_613E40`, with `flt_6140AC` below `flt_613E44` and `dword_61416C` 0: gear 0
//! and both drive speeds 0. By the look of it the engine stalling when the car turns
//! back on itself (a reading).
//!
//! In:  nothing
//! Out: esi 0x71A17C and edi 0x61409C when the dot was taken, else esi 0x61409C; the rest
//!      kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B896), every physics step.
//! Self-test: a copy of the original on test variables (its two calls kept), 12 cases
//! under three precisions.

use crate::errors::Refused;
use crate::framework::{
    EDI, ESI, RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva,
};
use crate::math::x87_run::{
    TEST_CONTROL_WORDS, abs_compare, compare_full, ext_div2, with_control_word,
};
use crate::physics::dot_product::dot_product;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::rounded;
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::stall_when_still", rva(0x440B9D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The kept motion (3), the scratch, the motion (3), the dot, the block, gear, speeds (2).
const VARIABLES: [usize; 12] = [
    0x61409C, 0x6140A0, 0x6140A4, 0x62400C, 0x71A17C, 0x71A180, 0x71A184, 0x6140AC, 0x61416C,
    0x71A1AC, 0x71A190, 0x71A194,
];

/// The Rust core on the variables at `v`; `r` as the original leaves it.
pub unsafe fn stall_at(v: &[u32], r: &mut Regs) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        r.esi = v[0];
        put(3, rounded(vector_length(v[0])));
        let zero: u32 = game_read(rva(0x5C5D90));
        if compare_full(get(3), zero) & 0x4000 == 0 {
            r.esi = v[4];
            r.edi = v[0];
            put(7, ext_div2(dot_product(v[4], v[0]), get(3), get(3)));
        }
        if abs_compare(get(3), game_read(rva(0x613E40))) & 0x100 != 0 {
            return;
        }
        if compare_full(get(7), game_read(rva(0x613E44))) & 0x100 == 0 {
            return;
        }
        if get(8) != 0 {
            return;
        }
        put(9, 0);
        put(10, 0);
        put(11, 0);
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version; `r` as the original leaves it.
pub unsafe fn stall_when_still(r: &mut Regs) {
    unsafe { stall_at(&game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { stall_when_still(&mut g) };
    Ok(Left::nothing().with(ESI, g.esi).with(EDI, g.edi))
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    let regions: Vec<_> = [3usize, 7, 9, 10, 11].iter().map(|&k| (v[k], 4)).collect();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; VARIABLES.len()];
    let base = cells.as_mut_ptr() as u32;
    let v: Vec<u32> = (0..VARIABLES.len() as u32).map(|k| base + 4 * k).collect();
    let variables: Vec<(usize, u32)> = VARIABLES.iter().copied().zip(v.iter().copied()).collect();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x440B9D,
            len: 0x9B,
            entry: &[0x8D, 0x35, 0x9C, 0x40, 0x61, 0x00],
            calls: &[0x6, 0x32],
            jumps_out: &[],
        },
        &|_| None,
        &variables,
        14,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 4 * cells.len())];
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..12u32 {
                let fill = || {
                    let kept = [
                        [0.0f32, 0.0, 0.0],
                        [3.0, 0.0, 4.0],
                        [0.01, 0.0, 0.0],
                        [-2.0, 1.0, 5.0],
                    ][(seed % 4) as usize];
                    let motion = [[3.0f32, 0.0, 4.0], [-3.0, 0.0, -4.0], [0.5, 0.1, -0.2]]
                        [(seed / 4 % 3) as usize];
                    let w = |k: usize, x: u32| unsafe { (v[k] as *mut u32).write(x) };
                    for k in 0..3 {
                        w(k, kept[k].to_bits());
                        w(4 + k, motion[k].to_bits());
                    }
                    w(3, 0xAAAA_AAAA);
                    w(7, 0xAAAA_AAAA);
                    w(8, u32::from(seed % 5 == 4));
                    w(9, 3);
                    w(10, 7.0f32.to_bits());
                    w(11, 8.0f32.to_bits());
                };
                fill();
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = capture(&all);
                fill();
                let mut g = Regs::SENTINEL;
                unsafe { stall_at(&v, &mut g) };
                let mine = capture(&all);
                let regs_ok = Left::nothing()
                    .with(ESI, g.esi)
                    .with(EDI, g.edi)
                    .matches(&Regs::SENTINEL, &ro);
                t.check(regs_ok && theirs == mine, || {
                    format!("control word {word:#x}, seed {seed}: regs_ok={regs_ok} original {theirs:x?}; rust {mine:x?}")
                });
            }
        });
    }
    drop((copy, cells));
    t.finish()
}
