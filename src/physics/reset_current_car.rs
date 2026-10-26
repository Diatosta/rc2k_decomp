//! `physics::reset_current_car` (0x43FE18): `physics::reset_car_grades` on the work
//! copy (`lea esi, g_car`).
//!
//! In:  nothing
//! Out: esi `g_car`; eax 0, ecx 0, edi `g_car` + 0x368; no flags read (`jmp` next)
//!
//! Caller: `sub_43BA8E` (0x43BA97, when `dword_71A234` is 2), under `sub_43EB5D`.
//! Self-test: a copy of the original whose `g_car` is a test block and whose call goes
//! to a copy of `reset_car_grades` on the same block.

use crate::errors::Refused;
use crate::framework::{ESI, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::reset_car_grades::{BLOCK_LEN, fill_block, reset_car_grades, reset_grades_at};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::reset_current_car",
    rva(0x43FE18),
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

const CAR_RVA: usize = rva(0x719FD8); // g_car

/// The Rust version.
pub unsafe fn reset_current_car() -> Left {
    let car = game_addr(CAR_RVA) as u32;
    unsafe { reset_car_grades(car) }.with(ESI, car)
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { reset_current_car() })
}

extern "C" fn dispatch(r: &mut Regs) {
    let car = (game_addr(CAR_RVA) as u32, BLOCK_LEN);
    dispatch_state(&HOOK, r, call_original, &[car], &rust);
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
    let mut block = vec![0u32; BLOCK_LEN / 4];
    let b = block.as_mut_ptr() as u32;
    let inner = match crate::physics::reset_car_grades::test_copy(b + 0x34, b + 0x30) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let target = inner.addr();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43FE18,
            len: 0xC,
            entry: &[0x8D, 0x35, 0xD8, 0x9F, 0x71, 0x00],
            calls: &[6],
            jumps_out: &[],
        },
        &|r| (r == rva(0x440A22)).then_some(target),
        &[(0x719FD8, b)],
        1,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    for seed in 1..=6u32 {
        fill_block(b, seed);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&[(b, BLOCK_LEN)]);
        fill_block(b, seed);
        let left = unsafe { reset_grades_at(b, b + 0x34, b + 0x30) }.with(ESI, b);
        let mine = capture(&[(b, BLOCK_LEN)]);
        let regs_ok = left.matches(&Regs::SENTINEL, &ro);
        t.check(regs_ok && theirs == mine, || {
            format!("seed {seed}: regs_ok={regs_ok}; the block differs")
        });
    }
    drop((copy, inner, block));
    t.finish()
}
