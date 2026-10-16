//! `race::sample_mud_colours` (0x44A0D6): 256 colours taken from a picture surface.
//!
//! In:  nothing
//! Out: eax the surface's number, ecx and edx the last place, edi past the colours, ebp
//!      0x100; the rest preserved
//!
//! See `race/mud.rs`. The surface's lock and unlock are the game's (DirectDraw). One
//! caller, `race::init_mud`. Self-test: a redirected copy whose lock and unlock note
//! themselves, surfaces of the test's (16 and 32 bits), the game's colours put back
//! afterwards.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::race::mud::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::sample_mud_colours", rva(0x44A0D6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { sample_mud_colours(r, &mut game_part) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !has_format() {
        return t.skip("no pixel format");
    }
    let copy = match copy(
        0x44A0D6,
        0x58,
        &[0xA1, 0xEC, 0x0D, 0x5D, 0x00],
        &[0x05, 0x3A, 0x52],
        &[(0x41AA4B, 2), (0x41AB02, 3)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [(game_addr(COLOURS_RVA) as u32, 0x400)];
    let saved = capture(&regions);
    for bits in [16u32, 32] {
        let surface = TestSurface::new(bits);
        run_both(
            &mut t,
            &format!("{bits} bits"),
            &copy,
            &regions,
            &|| unsafe {
                std::ptr::write_bytes(regions[0].0 as *mut u8, 0xAA, 0x400);
            },
            &|r| unsafe { sample_mud_colours(r, &mut test_part) },
        );
        drop(surface);
    }
    restore(&regions, &saved);
    t.finish()
}
