//! `physics::ease_car_value` (0x4554CA): unless `dword_6235E8` is set, `dword_71AF94`
//! follows `dword_71AF90` << 8: set to it when `dword_71AF88` is set, else stepped by 4
//! towards it (up when the difference's sign is clear, `jns`). What the values are is
//! not known.
//!
//! In:  nothing
//! Out: eax `dword_71AF90` << 8 (when `dword_6235E8` is 0); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B705), every physics step.
//! Self-test: a copy of the original on test variables, the switches and values above,
//! at and below, near the sign's turn.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::cells::{get, put_changed};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state};
use rc2k_core::physics::easing::ease_value;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::ease_car_value", rva(0x4554CA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The variables (IDA addresses): the block, the jump switch, the target, the value.
const VARIABLES: [usize; 4] = [0x6235E8, 0x71AF88, 0x71AF90, 0x71AF94];

/// The Rust version on the variables at `v`; `r` as the original leaves it.
// Converted: `rc2k_core::physics::easing::ease_value`.
pub unsafe fn ease_at(v: &[u32; 4], r: &mut Regs) {
    unsafe {
        let was = get(v[3]);
        let mut value = was;
        if let Some(eax) = ease_value(get(v[0]), get(v[1]), get(v[2]), &mut value) {
            r.eax = eax;
        }
        put_changed(v[3], was, value);
    }
}

fn game() -> [u32; 4] {
    VARIABLES.map(|x| game_addr(rva(x)) as u32)
}

/// The Rust version.
pub unsafe fn ease_car_value(r: &mut Regs) {
    unsafe { ease_at(&game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { ease_car_value(&mut g) };
    Ok(Left::nothing().with(EAX, g.eax))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(game()[3], 4)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 4];
    let base = cells.as_mut_ptr() as u32;
    let v: [u32; 4] = std::array::from_fn(|k| base + 4 * k as u32);
    let variables: Vec<(usize, u32)> = VARIABLES.iter().zip(v).map(|(&g, t)| (g, t)).collect();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4554CA,
            len: 0x44,
            entry: &[0x83, 0x3D, 0xE8, 0x35, 0x62, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &variables,
        8,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let all: [Region; 1] = [(base, 16)];
    for block in [0u32, 1] {
        for jump in [0u32, 5] {
            for (target, value) in [
                (0x10u32, 0x1000u32),
                (0x10, 0x0FFC),
                (0x10, 0x1004),
                (0x10, 0x1002),
                (0x7F_FFFF, 0x8000_0000),
                (0xFF_FFFF, 0),
                (0, 0x7FFF_FFFF),
            ] {
                let fill = || unsafe {
                    for (k, x) in [block, jump, target, value].into_iter().enumerate() {
                        (v[k] as *mut u32).write_volatile(x);
                    }
                };
                fill();
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = capture(&all);
                fill();
                let mut g = Regs::SENTINEL;
                unsafe { ease_at(&v, &mut g) };
                let mine = capture(&all);
                let regs_ok = Left::nothing()
                    .with(EAX, g.eax)
                    .matches(&Regs::SENTINEL, &ro);
                t.check(regs_ok && theirs == mine, || {
                    format!(
                        "block {block}, jump {jump}, target {target:#x}, value {value:#x}: regs_ok={regs_ok} original {theirs:x?}; rust {mine:x?}"
                    )
                });
            }
        }
    }
    drop((copy, cells));
    t.finish()
}
