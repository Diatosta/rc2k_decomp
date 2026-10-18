//! `physics::corner_hits_car` (0x45F703): whether a corner of another car (`dword_6D3BD4`
//! .., floats) lies inside the work car. In `pusha` / `popa`: the corner x 65536
//! (`flt_5C5D98`, unrounded) turned by `flt_6D3C40` (`scene::rotate_point`) and / 65536
//! into `flt_6D3BC8` ... Inside when each part's size is below the box's (`flt_6D3BBC`
//! ..; unordered counts as below, as `fcomp` / `jnb`): `dword_6D3BB4` = 1,
//! `physics::restore_saved_car` (in `pusha` / `popa`), `physics::bump_cars`.
//!
//! In:  edi the other car's block (passed to `bump_cars`)
//! Out: outside, every register kept; inside, as `bump_cars` leaves them; no flags read
//!      (the caller goes on)
//!
//! Caller: `physics::car_collisions` (eight corners a car).
//! Refuses what `restore_saved_car` and `bump_cars` refuse (what was written put back).
//! Self-test: `physics/car_collisions.rs` (`test_bump`): the original and the Rust version
//! on the game's variables (saved and put back) with test car blocks, corners inside and
//! out, under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::math::x87_run::{abs_compare, ext_times};
use crate::physics::car_collisions::{regions, test_bump};
use crate::physics::flow::{a, base, g, guard, p, x87_run};
use crate::scene::matrix3::extended;
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::corner_hits_car", rva(0x45F703), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Whether the corner is inside (`flt_6D3BC8` .. written either way).
pub fn corner_inside() -> bool {
    let k = g(0x5C5D98);
    let corner = [0x6D3BD4, 0x6D3BD8, 0x6D3BDC].map(|x| ext_times(extended(g(x)), k));
    let turned = unsafe { rotate_point(a(0x6D3C40), corner) };
    for (n, e) in turned.iter().enumerate() {
        x87_run!(
            "fld tbyte ptr [{e}]",
            "fdiv dword ptr [{b} + 0x1C5D98]",
            "fstp dword ptr [{o}]";
            b = base(), e = e.as_ptr(), o = a(0x6D3BC8 + 4 * n)
        );
    }
    (0..3).all(|n| abs_compare(g(0x6D3BC8 + 4 * n), g(0x6D3BBC + 4 * n)) & 0x100 != 0)
}

/// The Rust version.
pub unsafe fn corner_hits_car(entry: &Regs) -> Result<Left, Refused> {
    guard(&regions(), || {
        if !corner_inside() {
            return Ok(Left::nothing());
        }
        p(0x6D3BB4, 1);
        unsafe { crate::physics::restore_saved_car::restore_saved_car() }?;
        unsafe { crate::physics::bump_cars::bump_cars(entry.edi) }
    })
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { corner_hits_car(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    test_bump(&HOOK, call_original, &rust, true)
}
