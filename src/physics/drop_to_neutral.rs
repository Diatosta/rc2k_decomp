//! `physics::drop_to_neutral` (0x43FD71): with `dword_71A20C` set, in gear, the engine
//! speed below `flt_6140FC` (`fcomp`, C0: also when unordered) and `dword_61416C` 0: gear
//! 0, `dword_71A1A8` = `dword_613FA4`, `dword_71A1A4` = 0x14, `dword_71A210` and
//! `dword_71A214` 1 (as the gear changes of `sub_43C20D` set them). By the look of it the
//! engine stalling into neutral (a reading).
//!
//! In:  nothing
//! Out: every register kept (eax pushed around `fnstsw`)
//!
//! Caller: `sub_43B635` (0x43B966), every physics step.
//! Self-test: a copy of the original on test variables, every gate, speeds above, at,
//! below the limit and NaN.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::drop_to_neutral", rva(0x43FD71), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The variables' addresses (game RVAs as IDA shows them, in this order): the switch,
/// the gear, the engine speed, the limit, the block, the source, the four written.
const VARIABLES: [usize; 10] = [
    0x71A20C, 0x71A1AC, 0x71A1B0, 0x6140FC, 0x61416C, 0x613FA4, 0x71A1A8, 0x71A1A4, 0x71A210,
    0x71A214,
];

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust core on the variables at `v` (in `VARIABLES`' order).
pub unsafe fn drop_to_neutral_at(v: &[u32; 10]) {
    unsafe {
        if at(v[0]) == 0 || at(v[1]) == 0 {
            return;
        }
        if crate::math::x87::compare_status(at(v[2]), at(v[3])) & 0x100 == 0 {
            return;
        }
        if at(v[4]) != 0 {
            return;
        }
        put(v[1], 0);
        put(v[6], at(v[5]));
        put(v[7], 0x14);
        put(v[8], 1);
        put(v[9], 1);
    }
}

fn game() -> [u32; 10] {
    VARIABLES.map(|x| game_addr(rva(x)) as u32)
}

/// The Rust version.
pub unsafe fn drop_to_neutral() {
    unsafe { drop_to_neutral_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { drop_to_neutral() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = game();
    let regions: Vec<Region> = [1, 6, 7, 8, 9].map(|k| (v[k], 4)).to_vec();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 10];
    let base = cells.as_mut_ptr() as u32;
    let v: [u32; 10] = std::array::from_fn(|k| base + 4 * k as u32);
    let variables: Vec<(usize, u32)> = VARIABLES.iter().zip(v).map(|(&g, t)| (g, t)).collect();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43FD71,
            len: 0x64,
            entry: &[0x83, 0x3D, 0x0C, 0xA2, 0x71, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &variables,
        11,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let all: [Region; 1] = [(base, 40)];
    let limit = 800.0f32.to_bits();
    for switch in [0u32, 1] {
        for gear in [0u32, 1, 3, u32::MAX] {
            for speed in [0.0f32, 799.9, 800.0, 1200.0, -5.0, f32::NAN] {
                for block in [0u32, 1] {
                    let fill = || unsafe {
                        for (k, &a) in v.iter().enumerate() {
                            put(a, 0xD000_0000 | k as u32);
                        }
                        put(v[0], switch);
                        put(v[1], gear);
                        put(v[2], speed.to_bits());
                        put(v[3], limit);
                        put(v[4], block);
                    };
                    fill();
                    let mut ro = Regs::SENTINEL;
                    unsafe { call_game(&mut ro, copy.addr()) };
                    let theirs = capture(&all);
                    fill();
                    unsafe { drop_to_neutral_at(&v) };
                    let mine = capture(&all);
                    let regs_ok = ro.preserved_except(&Regs::SENTINEL, 0);
                    t.check(regs_ok && theirs == mine, || {
                        format!(
                            "switch {switch}, gear {gear:#x}, speed {speed}, block {block}: regs_ok={regs_ok} original {theirs:x?}; rust {mine:x?}"
                        )
                    });
                }
            }
        }
    }
    drop((copy, cells));
    t.finish()
}
