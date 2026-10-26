//! `race::clip_and_draw_face` (0x454187): the face's polygon cut by depth (when some
//! vertices are outside) and to the screen, and with three vertices or more drawn
//! (`scene::draw_polygon_in_game`), `g_polygonDrawn` 1 (see `race/course_faces.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its callers return)
//!
//! Callers: `sub_4537E9` (four places). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose drawing call goes to a stand-in (the clips real), against the
//! Rust version on the same; test polygons inside and across the screen; all put back
//! (skipped without a race).

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::course_faces::clip_and_draw_face;
use crate::race::draw_screen_quad::quad_regions;
use crate::race::level_init::{note, take_called};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::clip_and_draw_face", rva(0x454187), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Its draw in the game.
pub fn draw_part(_: u32, r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { crate::scene::draw_polygon::draw_polygon_in_game(r)? };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { clip_and_draw_face(r, &mut draw_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { clip_and_draw_face(r, &mut draw_part) }?;
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(rva(0x621480)) } == 0 {
        return t.skip("no race loaded");
    }
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x454187,
            len: 0x38,
            entry: &[0xA1, 0xF4, 0x40, 0x62, 0x00, 0x85, 0xC0, 0x74],
            calls: &[0x09, 0x17, 0x25],
            jumps_out: &[],
        },
        &|c| (c == rva(0x44BA2C)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let regions = quad_regions();
    let saved = capture(&regions);
    for (x, outside) in [(100.0f32, 0u32), (-50.0, 0), (300.0, 0)] {
        let prepare = || unsafe {
            restore(&regions, &saved);
            let entries = game_addr(rva(0x975510)) as u32;
            let vertices = game_addr(rva(0x623E40)) as u32;
            let list = game_addr(rva(0x977710)) as u32;
            for k in 0..4u32 {
                let e = entries + 0x44 * k;
                (e as *mut u32).write_unaligned(u32::MAX);
                ((e + 4) as *mut u32).write_unaligned(vertices + 0x20 * k);
                ((list + 4 * k) as *mut u32).write_unaligned(k);
                let v = vertices + 0x20 * k;
                ((v + 0xC) as *mut u32).write_unaligned((x + 80.0 * (k & 1) as f32).to_bits());
                ((v + 0x18) as *mut u32).write_unaligned(5000.0f32.to_bits());
                ((v + 0x1C) as *mut u32).write_unaligned((60.0 + 70.0 * (k >> 1) as f32).to_bits());
            }
            ((list + 16) as *mut u32).write_unaligned(0);
            game_write(rva(0x6240F8), 4u32);
            game_write(rva(0x6240F4), outside);
            game_write(rva(0x624114), 4u32);
            game_write(rva(0x6240A4), game_addr(rva(0x977F10)) as u32);
            take_called();
        };
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (take_called().len(), capture(&regions));
        prepare();
        let mut rr = Regs::SENTINEL;
        let done = unsafe {
            clip_and_draw_face(&mut rr, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (take_called().len(), capture(&regions));
        t.check(done.is_ok() && theirs == mine, || {
            let diff = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
            format!(
                "x {x}: calls {} / {}; memory first differs at {diff:x?}",
                theirs.0, mine.0
            )
        });
    }
    restore(&regions, &saved);
    t.finish()
}
