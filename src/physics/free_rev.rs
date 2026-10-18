//! `physics::free_rev` (0x43EA80): the engine revving free. `physics::engine_torque`;
//! `flt_614750` = `flt_614740` x `flt_5CB968` / gear 0's ratio / 10000 x +0x638 (`fild`) /
//! 256; the engine speed + it x the step / the ratio; the drag `dword_62400C` = engine
//! speed x `flt_613FD8` + `flt_613FD4`; then the engine speed brought toward 0 by the drag
//! (not past it; at 0 or unordered left alone).
//!
//! The x87 run is the original's instructions over the game's variables (from IDA).
//!
//! In:  nothing
//! Out: as `engine_torque` leaves them; no flags read (the callers return or call on)
//!
//! Callers: `physics::free_rev_in_neutral`, `physics::free_rev_clutch`.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back), `physics::engine_torque`'s cases, under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::math::x87;
use crate::physics::engine_torque::{below, engine_torque, g, p, regions, test_engine};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::free_rev", rva(0x43EA80), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// 0x43EA85 .. 0x43EAE7, as the original.
unsafe fn run() {
    let b = game_addr(0) as u32;
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x214740]",
            "fmul dword ptr [{b} + 0x1CB968]",
            "fdiv dword ptr [{b} + 0x2140C0]",
            "fdiv dword ptr [{b} + 0x1C5E34]",
            "fild dword ptr [{b} + 0x31A610]",
            ".byte 0xDE, 0xC9", // fmulp st(1), st
            "fdiv dword ptr [{b} + 0x1C5E2C]",
            "fstp dword ptr [{b} + 0x214750]",
            "fld dword ptr [{b} + 0x214750]",
            "fmul dword ptr [{b} + 0x21388C]",
            "fdiv dword ptr [{b} + 0x2140C0]",
            "fadd dword ptr [{b} + 0x31A1B0]",
            "fstp dword ptr [{b} + 0x31A1B0]",
            "fld dword ptr [{b} + 0x31A1B0]",
            "fmul dword ptr [{b} + 0x213FD8]",
            "fadd dword ptr [{b} + 0x213FD4]",
            "fstp dword ptr [{b} + 0x22400C]",
            b = in(reg) b,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

/// The Rust version.
pub unsafe fn free_rev() -> Left {
    let left = unsafe { engine_torque() };
    unsafe { run() };
    let (e, drag) = (g(0x71A1B0), g(0x62400C));
    let x = f32::from_bits(e);
    if x == 0.0 || x.is_nan() {
        return left;
    }
    if x < 0.0 {
        let y = x87::add(e, drag);
        p(0x71A1B0, y);
        if !below(y, 0) {
            p(0x71A1B0, 0);
        }
    } else {
        let y = x87::sub(e, drag);
        p(0x71A1B0, y);
        if below(y, 0) {
            p(0x71A1B0, 0);
        }
    }
    left
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { free_rev() })
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = regions();
    regions.push((game_addr(rva(0x71A1B0)) as u32, 4));
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_engine(&mut t, call_original, &rust);
    t.finish()
}
