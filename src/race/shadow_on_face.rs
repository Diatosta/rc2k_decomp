//! `race::shadow_on_face` (0x45773C): the car's shadow cut to one course face and drawn on
//! it (see `race/car_shadow.rs`): the hull's list as the polygon, `race::clip_shadow_to_face`,
//! each vertex raised onto the face (`sub_457B86`, covered here: `physics::face_shares` and
//! `physics::plane_height`), projected (`scene::project_polygon`, `scene::fog_polygon`),
//! cut by depth and to the screen, drawn in mode 5 (`scene::draw_polygon_in_game`).
//!
//! In:  nothing (the face's corners `dword_641938` ..)
//! Out: the registers as it leaves them (its caller calls it inside `pusha`)
//!
//! Refuses (before anything) a hull list past 0x40; what the projection and the clips
//! refuse (after the work is set). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose drawing call goes to a stand-in, the rest real, against the
//! Rust version on the same; the test square and faces of `clip_shadow_to_face`'s test;
//! all put back (skipped without a race: the projection needs the view).

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::race::car_shadow::*;
use crate::race::clip_shadow_to_face::{FACES, clip_regions, set_square};
use crate::race::level_init::{note, take_called};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::shadow_on_face", rva(0x45773C), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { shadow_on_face(r, &mut face_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { shadow_on_face(r, &mut face_part) }?;
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { crate::framework::game_read::<u32>(rva(0x5C6FB8)) } == 0
        || unsafe { crate::framework::game_read::<u32>(rva(0x621480)) } == 0
    {
        return t.skip("no race loaded");
    }
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45773C,
            len: 0x143,
            entry: &[0xC7, 0x05, 0x34, 0x41, 0x62, 0x00, 0x00, 0x00],
            calls: &[0x8E, 0xAC, 0xBA, 0xBF, 0xE1, 0xEF, 0x12F],
            jumps_out: &[],
        },
        &|c| {
            FACE_CALLEES
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
    let mut points = vec![0u32; 3 * 0x80];
    let p = points.as_mut_ptr() as u32;
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let mut regions = clip_regions();
    regions.extend([
        (p, 12 * 0x80),
        g(0x975510, 0x4AF0),
        g(0x6240A4, 0x94),
        at(rva(0x641990)),
        at(rva(0x63DAB4)),
        g(0x6396FC, 0x120),
        g(0x639800, 0x20),
        g(0x639890, 0xC),
    ]);
    let records: u32 = unsafe { crate::framework::game_read(rva(0x5C6FB8)) };
    regions.push((records, 0x20 * 0x60));
    let mut kept = regions.clone();
    kept.extend([
        at(rva(0x5CCCC0)),
        at(rva(0x5CCD20)),
        g(0x64199C, 0x104),
        g(0x641938, 0xC),
        g(0x71A10C, 0xC),
        at(rva(0x641930)),
        at(rva(0x641928)),
    ]);
    let saved = capture(&kept);
    for (n, face) in FACES.iter().enumerate() {
        let prepare = || {
            restore(&kept, &saved);
            let pts = unsafe { std::slice::from_raw_parts_mut(p as *mut u32, 3 * 0x80) };
            set_square(pts, *face, (0x5000, 0x5000));
            unsafe {
                game_write(rva(0x5CCD20), 0u32);
                game_write(rva(0x641A9C), 4u32);
                for k in 0..4u32 {
                    game_write(rva(0x64199C + 4 * k as usize), k + 1);
                }
                game_write(rva(0x641930), 0xC3u32);
                game_write(rva(0x641928), 0x1999u32);
            }
            take_called();
        };
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (take_called(), capture(&regions));
        prepare();
        let done = unsafe {
            shadow_on_face(&Regs::SENTINEL, &mut |pl, r| {
                note(pl, r);
                Ok(())
            })
        };
        let mine = (take_called(), capture(&regions));
        let clipped =
            crate::race::car_shadow::SCREEN_CLIPPED.load(std::sync::atomic::Ordering::SeqCst);
        let same = if clipped {
            theirs.1 == mine.1 && theirs.0.iter().map(|c| c.0).eq(mine.0.iter().map(|c| c.0))
        } else {
            theirs == mine
        };
        t.check(done.is_ok() && same, || {
            let diff = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
            format!("face {n}: original calls {:x?}; rust {:x?} {:?}; memory first differs at {diff:x?}", theirs.0, mine.0, done.as_ref().err())
        });
    }
    restore(&kept, &saved);
    drop(points);
    t.finish()
}
