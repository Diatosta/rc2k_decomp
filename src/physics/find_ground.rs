//! `physics::find_ground` (0x441E29): `call physics::ground_search; retn`.
//!
//! In:  the point (`dword_639824`, `dword_63982C`), the face mask `dword_620470`, the
//!      last face `dword_5CCC78`
//! Out: every register kept; `g_groundFound`, `dword_5CCC74`, the face's shares and
//!      height (`dword_639828`); CF not read by its callers
//!
//! See `physics/ground.rs`. Callers (7): `sub_45D305` (the car's ground points),
//! `sub_462485`, `sub_45EB06`, `sub_458078`, `sub_44ECE4`, `sub_43A1CA`, `sub_438E22`.
//! Refuses what `ground_search` refuses.
//! Self-test: a copy of the original (its call to a copy of `ground_search`) on the
//! test's ground, as `ground_search`'s test.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::ground::*;
use crate::physics::ground_search::{ground_search, test_ground};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::find_ground", rva(0x441E29), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the carry (the registers are kept).
pub unsafe fn find_ground() -> Result<bool, Refused> {
    unsafe { ground_search() }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { find_ground() }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &Ground::game().regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let test = test_ground(3);
    let copies = match test.copies() {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let g = test.ground();
    let mut found = 0;
    for (n, &(x, z)) in test_points().iter().enumerate() {
        let prepare = || unsafe {
            test.clear_work();
            test.set_point(x, z, if n % 3 == 0 { 0x80 } else { 0 }, n % 2 == 1);
            g.put(0x5CCC78, u32::MAX);
        };
        if check_case(
            &mut t,
            &format!("point ({x:#x}, {z:#x})"),
            copies.find_ground.addr(),
            &test,
            Regs::SENTINEL,
            &prepare,
            &|g, r| unsafe { ground_search_at(g, r) },
            true,
        ) {
            found += 1;
        }
    }
    t.check(found > 0, || "no case found the point".into());
    drop(copies);
    t.finish()
}
