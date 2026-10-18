//! `race::race_from_menu` (0x485570): the menu's 3D memory tidied (`race::render3d_tidy`,
//! `race::objects_tidy`, `nullsub_6`), the textures released (`race::release_textures`),
//! then the race (`race::start_race`).
//!
//! In:  nothing
//! Out: the registers as the race leaves them
//!
//! See `race/race_end.rs`. One caller, `sub_4843EC` (a call follows). It begins with its
//! call: private copy, its five calls relocated, manual patch. Steps through
//! `or_in_game`. Shadow mode runs the original alone (it runs the race). Self-test: a
//! copy whose four calls go to stand-ins (`nullsub_6` the real one), against the Rust
//! version on the same.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::race_end::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::race_from_menu", rva(0x485570), thunk, &ORIGINAL)
    .private_copy(0x1A)
    .relocate_calls(&[0x0, 0x5, 0xA, 0xF, 0x14])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    // It runs a whole race: a gated thunk would count the game's thread as inside a
    // hook throughout, and every test's pause wait out its 2 s (2026-10-18).
    ungated
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { race_from_menu(r, &mut menu_part) } {
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
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x485570,
        0x1A,
        &[0xE8, 0x32, 0xDE, 0xFB, 0xFF],
        &[0x0, 0x5, 0xA, 0xF, 0x14],
        &[(0x4433A7, 1), (0x44346B, 2), (0x443396, 3), (0x4278B7, 4)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [crate::service::harness::at(rva(0x621870))];
    let saved = crate::service::harness::capture(&regions);
    run_both(
        &mut t,
        "the calls",
        &copy,
        &regions,
        &|| unsafe { crate::framework::game_write(rva(0x621870), 0x5A5A_5A5Au32) },
        &|r| unsafe { race_from_menu(r, &mut test_part) },
    );
    crate::service::harness::restore(&regions, &saved);
    t.finish()
}
