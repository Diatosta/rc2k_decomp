//! `physics::update_car_angles` (0x43BEDC): `physics::car_angles` when `dword_71A6C8` is
//! set or `dword_71A22C` is 0; else nothing.
//!
//! In:  nothing
//! Out: when run, esi = `g_crashMatrix`; the rest kept; no flags read (the callers' next
//!      instructions are `lea` and `mov`)
//!
//! Callers: `sub_43B635` (0x43B83B, 0x43B8B4, after `race::invert_car_matrix`).
//! Self-test: a copy of the original (its switches the test's, its call to
//! `car_angles`' test copy) on the four switch settings.

use crate::errors::Refused;
use crate::framework::{ESI, RawHook, Regs, SelfTest, call_game, game_read, hook_asm, rva};
use crate::physics::car_angles::{car_angles, car_angles_at, regions};
use crate::physics::matrix_angles::AngleTest;
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::update_car_angles",
    rva(0x43BEDC),
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

const FORCE_RVA: usize = rva(0x71A6C8); // dword_71A6C8
const HOLD_RVA: usize = rva(0x71A22C); // dword_71A22C

/// Whether the angles are made, by the two switches.
pub fn wanted(force: u32, hold: u32) -> bool {
    force != 0 || hold == 0
}

/// The Rust version.
pub unsafe fn update_car_angles() -> Left {
    let (force, hold) = unsafe { (game_read::<u32>(FORCE_RVA), game_read::<u32>(HOLD_RVA)) };
    if wanted(force, hold) {
        unsafe { car_angles() }
    } else {
        Left::nothing()
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { update_car_angles() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let test = match AngleTest::new() {
        Ok(x) => x,
        Err(why) => return t.skip(&why),
    };
    let inner = match crate::physics::car_angles::test_copy(&test) {
        Ok(x) => x,
        Err(why) => return t.skip(&why),
    };
    let target = inner.addr();
    let sw = test.switches;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43BEDC,
            len: 0x18,
            entry: &[0x83, 0x3D, 0xC8, 0xA6, 0x71, 0x00, 0x00],
            calls: &[0x12],
            jumps_out: &[],
        },
        &|r| (r == rva(0x43BEF4)).then_some(target),
        &[(0x71A6C8, sw), (0x71A22C, sw + 4)],
        2,
    ) {
        Ok(x) => x,
        Err(why) => return t.skip(&why),
    };
    for (n, (force, hold)) in [(0u32, 0u32), (0, 1), (1, 0), (0x8000_0000, 5)]
        .into_iter()
        .enumerate()
    {
        let set = || unsafe {
            test.fill(2 * n as u32 + 2);
            (sw as *mut [u32; 2]).write_volatile([force, hold]);
        };
        set();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&test.all());
        set();
        let left = if wanted(force, hold) {
            unsafe { car_angles_at(test.m, &test.places, test.ints) };
            Left::nothing().with(ESI, test.m)
        } else {
            Left::nothing()
        };
        let mine = capture(&test.all());
        let regs_ok = left.matches(&Regs::SENTINEL, &ro);
        t.check(regs_ok && theirs == mine, || {
            format!("switches {force:#x}, {hold:#x}: regs_ok={regs_ok}; the memory differs")
        });
    }
    drop((copy, inner, test));
    t.finish()
}
