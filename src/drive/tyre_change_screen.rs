//! `drive::tyre_change_screen` (0x445047): the tyre-change screen while a tyre change is
//! under way (the first block's +0x26C, `g_carBlockStore`, not 0 or -1):
//! `drive::tyre_screen` for the mode `dword_6238A4`, car 0.
//!
//! In:  nothing
//! Out: eax the mode, esi 0 when drawn; the rest kept; no flags read
//!
//! Caller: `sub_45429B` (the race's frame, stream A's).
//!
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go to
//! stand-ins noting them (`drive/tyre_parts.rs`), against the Rust version with the same.

use crate::drive::tyre_parts::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::{a, g, p};
use crate::race::car_starts::Part;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::tyre_change_screen", rva(0x445047), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { tyre_change_screen(r, &mut game_part) } {
        Ok(g) => {
            let esp = r.esp;
            *r = Regs {
                eflags: r.eflags,
                ..g
            };
            r.esp = esp;
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version, the screen through `part`.
// Converted: `rc2k_core::drive::race_ticks::tyre_change_shown`.
pub unsafe fn tyre_change_screen(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if !rc2k_core::drive::race_ticks::tyre_change_shown(g(0x70F3EC + 0x26C)) {
        return Ok(r);
    }
    r.eax = g(0x6238A4);
    r.esi = 0;
    let mut s = r;
    part(SCREEN, &mut s)?;
    Ok(r)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x445047,
        0x22,
        &[0x83, 0x3D, 0x58, 0xF6, 0x70, 0x00, 0x00],
        &[0x1C],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut all = test_regions();
    all.extend([(a(0x70F3EC + 0x26C), 4), (a(0x6238A4), 4)]);
    let saved = capture(&all);
    for change in [0u32, 1, u32::MAX, 0x40] {
        let fill = || {
            p(0x70F3EC + 0x26C, change);
            p(0x6238A4, 2);
        };
        both(
            &mut t,
            &format!("change {change:#x}"),
            &copy,
            Regs::SENTINEL,
            &fill,
            tyre_change_screen,
        );
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
