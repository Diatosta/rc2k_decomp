//! `race::load_stage` (0x444040): the stage's whole loading ("SHAUNINITEVERYTHING"), see
//! `race/stage_loading.rs`.
//!
//! In:  nothing
//! Out: eax = `g_sceneClock`, carry clear (`clc`); the rest as race init and the clock
//!      leave them
//!
//! Callers: seven (0x41CE94, 0x443F68, 0x443FC3, 0x444020, 0x459304, 0x4636E3,
//! 0x463D1F); each calls on at once (no flags read; carry cleared anyway). Its steps are
//! the Rust versions but race init (`sub_4462B9`), called in the game (section 15) until
//! it is ported; a step whose Rust version refuses runs in the game through its hook. It
//! begins with its call: private copy, its 19 calls relocated, manual patch. Shadow mode
//! runs the original alone (it logs and loads the stage). Self-test: a copy whose calls go
//! to stand-ins that note them, its ten references to `g_logIndent` the test's, against
//! the Rust version on the same: replay on and off, `dword_64BB18` on and off.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::stage_loading::*;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

const CALLS: [usize; 19] = [
    0x0, 0xC, 0x19, 0x26, 0x3E, 0x4D, 0x62, 0x71, 0x86, 0x9E, 0xA5, 0xBA, 0xC9, 0xDE, 0xED, 0xF2,
    0x215, 0x222, 0x228,
];

pub static HOOK: RawHook = RawHook::new("race::load_stage", rva(0x444040), thunk, &ORIGINAL)
    .private_copy(0x22F)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

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
    let indent = crate::framework::game_addr(rva(0x4E00A0)) as u32; // g_logIndent
    match unsafe { load_stage(r, indent, &mut game_part) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            };
            r.set_carry(false);
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::audio::test_copy::{Copy, make};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let indent = Box::new(0u32);
    let at = &*indent as *const u32 as u32;
    let copy = match make(
        &HOOK,
        &Copy {
            address: 0x444040,
            len: 0x22F,
            entry: &[0xE8, 0x99, 0xD3, 0xFB, 0xFF],
            calls: &CALLS,
            jumps_out: &[],
        },
        &redirect,
        &[(0x4E00A0, at)],
        10,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    crate::install::stand_ins::set_handler(crate::race::level_init::note);
    let mut regions: Vec<Region> = regions();
    regions.push((at, 4));
    let saved = capture(&regions);
    for (replay, no_place, clock) in [
        (0u32, 0u32, 0x1234u32),
        (1, 0, 7),
        (0, 1, 0xFFFF_FFFF),
        (2, 3, 0),
    ] {
        crate::race::race_cars::run_cars(
            &mut t,
            &format!("replay {replay}, no place {no_place}, clock {clock:#x}"),
            &copy,
            &regions,
            Regs::SENTINEL,
            &|| {
                prepare(replay, no_place, clock);
                unsafe { (at as *mut u32).write(0x30) };
            },
            &|r| unsafe {
                load_stage(r, at, &mut |c, r| {
                    crate::race::level_init::note(c + 1, r);
                    Ok(())
                })
            },
        );
    }
    restore(&regions, &saved);
    drop(indent);
    t.finish()
}
