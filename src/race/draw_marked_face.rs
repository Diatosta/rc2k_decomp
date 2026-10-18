//! `race::draw_marked_face` (0x44CD90): `scene::scale_texture_coordinates`,
//! `scene::fog_polygon`, the drawing (`scene::draw_marked_polygon` with
//! `scene::draw_polygon_in_game`), `g_polygonDrawn` 1 (see `race/course_faces.rs`).
//!
//! In:  nothing
//! Out: the registers as the parts leave them (its caller `sub_453215` returns)
//!
//! It begins with its call: private copy (0x1A, its three calls relocated), manual patch.
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose drawing call
//! goes to a stand-in (the scaling and fade real), against the Rust version on the same;
//! a test quad; all put back (skipped without a race).

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva, shadow,
};
use crate::race::course_faces::*;
use crate::race::draw_screen_quad::quad_regions;
use crate::race::level_init::{note, take_called};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::draw_marked_face", rva(0x44CD90), thunk, &ORIGINAL)
    .private_copy(0x1A)
    .relocate_calls(&[0x00, 0x05, 0x0A])
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
    if let Err(e) = unsafe { draw_marked_face(r, &mut marked_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
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
            address: 0x44CD90,
            len: 0x1A,
            entry: &[0xE8, 0x66, 0x15, 0x00, 0x00, 0xE8, 0xC0, 0x14],
            calls: &[0x00, 0x05, 0x0A],
            jumps_out: &[],
        },
        &|c| (c == rva(0x44B9FC)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let regions = quad_regions();
    let saved = capture(&regions);
    for count in [3u32, 4] {
        let prepare = || unsafe {
            restore(&regions, &saved);
            let entries = game_addr(rva(0x975510)) as u32;
            let vertices = game_addr(rva(0x623E40)) as u32;
            let list = game_addr(rva(0x977710)) as u32;
            for k in 0..count {
                let e = entries + 0x44 * k;
                (e as *mut u32).write_unaligned(u32::MAX);
                ((e + 4) as *mut u32).write_unaligned(vertices + 0x20 * k);
                ((e + 8) as *mut u32).write_unaligned((k as f32 * 0.3).to_bits());
                ((e + 0xC) as *mut u32).write_unaligned((1.0 - k as f32 * 0.3).to_bits());
                ((e + 0x20) as *mut u8).write(0x40);
                ((list + 4 * k) as *mut u32).write_unaligned(k);
                let v = vertices + 0x20 * k;
                ((v + 0xC) as *mut u32).write_unaligned((100.0 + 50.0 * k as f32).to_bits());
                ((v + 0x18) as *mut u32).write_unaligned(5000.0f32.to_bits());
                ((v + 0x1C) as *mut u32).write_unaligned((80.0 + 30.0 * (k & 1) as f32).to_bits());
            }
            crate::framework::game_write(rva(0x6240F8), count);
            take_called();
        };
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (take_called(), capture(&regions));
        prepare();
        let mut rr = Regs::SENTINEL;
        let done = unsafe {
            draw_marked_face(&mut rr, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (take_called(), capture(&regions));
        t.check(
            done.is_ok() && theirs.1 == mine.1 && theirs.0.len() == mine.0.len(),
            || {
                let diff = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
                format!(
                    "vertices {count}: calls {} / {}; memory first differs at {diff:x?}",
                    theirs.0.len(),
                    mine.0.len()
                )
            },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
