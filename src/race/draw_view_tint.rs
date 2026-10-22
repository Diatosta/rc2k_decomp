//! `race::draw_view_tint` (0x456092): the whole view (0 .. `dword_62408C`, 0 ..
//! `dword_624090`) as one screen quad in 0x204080, by `race::draw_screen_quad` (see
//! `race/frame_draw.rs`).
//!
//! In:  nothing
//! Out: eax 0x204080; the rest kept (a `retn` follows)
//!
//! Caller: `sub_4557AB` (0x455890). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose quad call goes to a copy of `draw_screen_quad` with its
//! drawing calls stood in, against the Rust version on the same; view sizes; all put back.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::race::draw_screen_quad::{quad_inputs, quad_regions};
use crate::race::frame_draw::*;
use crate::race::level_init::{note, take_called};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_view_tint", rva(0x456092), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { draw_view_tint(r, &mut quad_part) } {
        Ok(after) => r.eax = after.eax,
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let after = unsafe { draw_view_tint(r, &mut quad_part) }?;
        r.eax = after.eax
    }
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let quad = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x44B364,
            len: 0x1D4,
            entry: &[0xC7, 0x05, 0x34, 0x41, 0x62, 0x00, 0x00, 0x00],
            calls: &[0x170, 0x17E, 0x1AB, 0x1C4],
            jumps_out: &[],
        },
        &|c| {
            QUAD_CALLEES
                .iter()
                .find(|&&(_, k)| rva(k) == c)
                .map(|&(p, _)| crate::install::stand_ins::stand_in(p))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let quad_addr = quad.addr();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x456092,
            len: 0x58,
            entry: &[0xB8, 0x00, 0x00, 0x00, 0x00, 0xA3, 0x10, 0x3E],
            calls: &[0x52],
            jumps_out: &[],
        },
        &|c| (c == rva(0x44B364)).then_some(quad_addr),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let mut regions = quad_regions();
    regions.extend(quad_inputs());
    let mut kept = regions.clone();
    kept.push((game_addr(rva(0x62408C)) as u32, 8));
    kept.push(at(rva(0x6240BC)));
    // The screen clip's bounds (`flt_6241E4` .. `flt_6241F0`): the test's, wider than any
    // view, so that the quad is never cut (the current screen's bounds cut it once, a path
    // whose registers are not modelled: 2026-10-18).
    kept.push((game_addr(rva(0x6241E4)) as u32, 0x10));
    let saved = capture(&kept);
    for (w, h) in [(640u32, 480u32), (320, 200), (0, 0)] {
        let prepare = || unsafe {
            restore(&kept, &saved);
            game_write(rva(0x62408C), w);
            game_write(rva(0x624090), h);
            for (n, bound) in [0.0f32, 4096.0, 0.0, 4096.0].into_iter().enumerate() {
                // top, bottom, left, right
                game_write(rva(0x6241E4) + 4 * n, bound.to_bits());
            }
            take_called();
        };
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (ro.eax, take_called(), capture(&regions));
        prepare();
        let done = unsafe {
            draw_view_tint(&Regs::SENTINEL, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (
            done.as_ref().map(|r| r.eax).unwrap_or(0),
            take_called(),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!("view {w} x {h}: original {:x} {:x?}; rust {:x} {:x?}; memory first differs at {diff:x?}", theirs.0, theirs.1, mine.0, mine.1)
        });
    }
    restore(&kept, &saved);
    drop(quad);
    t.finish()
}
