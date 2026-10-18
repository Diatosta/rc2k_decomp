//! `race::face_colours` (0x452887): the corners' colours (`g_lightTable` by the face's
//! +0x14 .., with flag 4 plus the two extra lights, `add_colours` MMX or not) and fog
//! bytes, then the face drawn (`race::draw_face`); see `race/course_faces.rs`.
//!
//! In:  esi = the face
//! Out: the registers as `draw_face` leaves them (callers return or go on with `xor`)
//!
//! Callers: `sub_452393` (three). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose `draw_face` call goes to a stand-in, against the Rust version
//! with the same: test faces, flag 4 on and off, MMX on and off.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::course_faces::*;
use crate::race::face_stage::compare_plain;
use crate::race::level_init::note;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::face_colours", rva(0x452887), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { face_colours(r, &mut |_, r| draw_face(r, &mut face_part)) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x452887,
            len: 0x2D1,
            entry: &[0x33, 0xC0, 0x0F, 0xB6, 0x46, 0x14, 0x8B, 0x04],
            calls: &[0x2CB],
            jumps_out: &[],
        },
        &|c| {
            [(0x452C6F, 1)]
                .iter()
                .find(|(a, _)| rva(*a) == c)
                .map(|(_, p)| crate::install::stand_ins::stand_in(*p))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let regions: Vec<Region> = vec![
        (game_addr(rva(0x63DA44)) as u32, 0x10),
        (game_addr(rva(0x63DA64)) as u32, 0x10),
    ];
    let mut kept = regions.clone();
    kept.push((game_addr(rva(0x5183A8)) as u32, 4));
    let saved = capture(&kept);
    let mut face = vec![0u8; 0x40];
    for (k, b) in face.iter_mut().enumerate() {
        *b = (k as u8).wrapping_mul(37).wrapping_add(11);
    }
    let f = face.as_mut_ptr() as u32;
    for (flag, mmx) in [(0u16, 1u32), (4, 1), (4, 0), (0xFFFF, 0)] {
        let line = compare_plain(
            &format!("flag {flag:#x} mmx {mmx}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x5183A8), mmx);
                ((f + 2) as *mut u16).write_unaligned(flag);
            },
            &regions,
            copy.addr(),
            Regs {
                esi: f,
                ..Regs::SENTINEL
            },
            &mut |r| unsafe {
                face_colours(r, &mut |p, r| {
                    note(p, r);
                    Ok(())
                })
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&kept, &saved);
    drop(face);
    t.finish()
}
