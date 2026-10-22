//! `race::draw_screen_quad` (0x44B364): a flat rectangle on the screen in
//! `dword_623DE8`'s colour, from x `dword_623E10` .. `dword_623E14` and y `dword_623E20` ..
//! `dword_623E24` (see `race/frame_draw.rs`): the polygon's four work entries and
//! vertices set, clipped (`scene::clip_polygon_to_screen`), drawn with the depth buffer
//! off when three vertices or more are left.
//!
//! In:  nothing
//! Out: every register kept by the Rust version (the original's change; all four callers
//!      return at once)
//!
//! In the game the depth buffer and the drawing by `scene::depth_buffer_on`'s and
//! `scene::draw_polygon_in_game`'s Rust versions. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose depth-buffer and drawing calls go to stand-ins, the
//! clip real, against the Rust version on the same; rectangles inside, across and off the
//! screen; all put back.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::race::frame_draw::*;
use crate::race::level_init::{note, take_called};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_screen_quad", rva(0x44B364), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { draw_screen_quad(r, &mut quad_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_screen_quad(r, &mut quad_part) }?;
    Ok(())
}

pub fn quad_regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        g(0x975510, 0x4AF0),
        g(0x623E40, 0x80),
        g(0x6240A4, 0x94),
        at(rva(0x63DAB4)),
    ]
}

pub fn quad_inputs() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![g(0x623DE8, 4), g(0x623E10, 0x18)]
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
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
    let regions = quad_regions();
    let mut kept = regions.clone();
    kept.extend(quad_inputs());
    let saved = capture(&kept);
    for (x, y) in [
        ((10u32, 200u32), (20u32, 100u32)),
        ((0xFFFF_FFC0, 50), (5, 3000)),
        ((5000, 6000), (5000, 6000)),
    ] {
        let prepare = || unsafe {
            restore(&kept, &saved);
            game_write(rva(0x623E10), x.0);
            game_write(rva(0x623E14), x.1);
            game_write(rva(0x623E20), y.0);
            game_write(rva(0x623E24), y.1);
            game_write(rva(0x623DE8), 0x0040_8020u32);
            take_called();
        };
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (take_called(), capture(&regions));
        // A clipped polygon: the clip's passes leave registers the Rust version does not
        // model (no callee reads them): the calls are compared by place alone.
        let clipped = unsafe { crate::framework::game_read::<u32>(rva(0x624114)) } != 4;
        prepare();
        let done = unsafe {
            draw_screen_quad(&Regs::SENTINEL, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (take_called(), capture(&regions));
        let same = if clipped {
            theirs.1 == mine.1 && theirs.0.iter().map(|c| c.0).eq(mine.0.iter().map(|c| c.0))
        } else {
            theirs == mine
        };
        t.check(done.is_ok() && same, || {
            let diff = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
            format!("x {x:x?} y {y:x?}: original calls {:x?}; rust {:x?} {done:?}; memory first differs at {diff:x?}", theirs.0, mine.0)
        });
    }
    restore(&kept, &saved);
    t.finish()
}
