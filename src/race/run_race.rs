//! `race::run_race` (0x443F0B): a race, start to end: the settings in, the arcade's cars,
//! the start states, the stage loaded (`race::load_stage`), the replay opened, the race
//! run (`sub_444CEE`, stream B's: in the game until ported), the replay closed,
//! everything tidied, the settings out. An arcade race's first stage draws a new noise
//! seed first.
//!
//! In:  nothing
//! Out: the registers as the last step leaves them
//!
//! See `race/race_end.rs`. One caller, `sub_4278B7` (a `retn` follows). Its steps are the
//! Rust versions but the race's run (section 15); a step that refuses runs in the game
//! through its hook. Shadow mode runs the original alone (it runs the race). Self-test: a
//! copy whose eleven calls go to stand-ins, its one reference to `g_logIndent` the test's,
//! against the Rust version on the same: arcade first stage and not.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::race_end::*;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::run_race", rva(0x443F0B), thunk, &ORIGINAL).self_test(self_test);

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
    let indent = game_addr(rva(0x4E00A0)) as u32; // g_logIndent
    match unsafe { run_race(r, indent, &mut race_part) } {
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
    use crate::audio::test_copy::{Copy, make};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let indent = Box::new(0u32);
    let at = &*indent as *const u32 as u32;
    let copy = match make(
        &HOOK,
        &Copy {
            address: 0x443F0B,
            len: 0x8B,
            entry: &[0xC7, 0x05, 0x1C, 0x10, 0x60, 0x00, 0x01, 0x00, 0x00, 0x00],
            calls: &[
                0x26, 0x44, 0x49, 0x58, 0x5D, 0x62, 0x67, 0x76, 0x7B, 0x80, 0x85,
            ],
            jumps_out: &[],
        },
        &|c| {
            RACE_CALLEES
                .iter()
                .position(|&k| k != 0 && rva(k) == c)
                .map(|n| crate::install::stand_ins::stand_in(n as u32))
        },
        &[(0x4E00A0, at)],
        1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    crate::install::stand_ins::set_handler(crate::race::level_init::note);
    let mut regions: Vec<Region> = race_regions();
    let saved = capture(&regions);
    regions.push((at, 4));
    for (first, mode) in [(0u32, 2u32), (1, 2), (0, 0), (3, 1)] {
        crate::race::mud::run_both(
            &mut t,
            &format!("stage in rally {first}, mode {mode}"),
            &copy,
            &regions,
            &|| unsafe {
                for &(a, len) in &regions {
                    std::ptr::write_bytes(a as *mut u8, 0x5A, len);
                }
                game_write(rva(0x75AFF4), first);
                game_write(rva(0x75ACD8), mode);
            },
            &|r| unsafe { run_race(r, at, &mut crate::race::mud::test_part) },
        );
    }
    restore(&race_regions(), &saved);
    drop(indent);
    t.finish()
}
