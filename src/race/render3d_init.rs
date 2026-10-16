//! `race::render3d_init` (0x4433A1): `scene::allocate_point_records` (`render3d init`; a `call` and a `retn`).
//!
//! In:  nothing
//! Out: as `scene::allocate_point_records` leaves them
//!
//! See `race/level_init.rs`. One caller, `sub_44426F` (`race::load_stage`'s
//! "loadlevel_init"). Shadow mode runs the original alone. Self-test: a redirected copy
//! whose variables and memory are the test's and whose calls note themselves.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::level_init::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::render3d_init", rva(0x4433A1), thunk, &ORIGINAL)
    .private_copy(0x6)
    .relocate_calls(&[0x00])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs, _: &Vars) -> Result<Regs, crate::errors::Refused> {
    let mut g = *r;
    game_part(2, &mut g)?;
    Ok(g)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match rust(r, &Vars::game()) {
        Ok(left) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..left
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let m = TestMemory::new();
    let copy = match copy_of(
        &m,
        0x4433A1,
        0x6,
        &[0xE8, 0xD0, 0xC5, 0xFD, 0xFF],
        &[0x00],
        0,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let v = m.vars();
    let put = |at: u32, value: u32| unsafe { (at as *mut u32).write_unaligned(value) };
    let _ = (&v, &put);
    both(&mut t, "the call", &copy, &m, &|| {}, &|r| {
        let mut g = *r;
        test_part(2, &mut g)?;
        Ok(g)
    });
    t.finish()
}
