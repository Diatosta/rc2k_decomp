//! `race::draw_face_sides` (0x452393): the face's front and, two-sided, its back, each
//! unless facing away (outside a race both), after `race::face_materials`; see
//! `race/course_faces.rs`.
//!
//! In:  esi = the face
//! Out: the registers as it leaves them (its caller goes on with a call)
//!
//! Caller: `sub_45232A`. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose two drawing calls go to stand-ins (`face_materials` real), against the Rust
//! version with the same: test faces with and without objects, one- and two-sided, each
//! facing, in a race and not.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::course_faces::*;
use crate::race::face_stage::compare_plain;
use crate::race::level_init::note;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_face_sides", rva(0x452393), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { draw_face_sides(r, &mut sides_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_face_sides(r, &mut sides_part) }?;
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x452393,
            len: 0xB1,
            entry: &[0x83, 0x3D, 0x80, 0x14, 0x62, 0x00, 0x00, 0x75],
            calls: &[0x09, 0x28, 0x41, 0x68, 0x6F, 0x76, 0x8A, 0xAA],
            jumps_out: &[],
        },
        &|c| {
            [(0x452887, 1), (0x452B58, 2)]
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
        (game_addr(rva(0x63D9E4)) as u32, 8),
        (game_addr(rva(0x63DA08)) as u32, 4),
        (game_addr(rva(0x63DA2C)) as u32, 0x18),
        (game_addr(rva(0x62400C)) as u32, 4),
    ];
    let mut kept = regions.clone();
    kept.push((game_addr(rva(0x621480)) as u32, 4));
    let saved = capture(&kept);
    let mut face = vec![0u8; 0x40];
    let f = face.as_mut_ptr() as u32;
    for (race, kind, objects, away) in [
        (0u32, 0u8, (1u8, 0u8), (1u32, 1u32)),
        (1, 0, (0, 0), (0, 0)),
        (1, 0, (1, 2), (0, 1)),
        (1, 0, (1, 2), (1, 1)),
        (1, 8, (1, 2), (0, 0)),
        (1, 8, (0, 2), (1, 1)),
        (1, 8, (1, 0), (1, 0)),
    ] {
        let line = compare_plain(
            &format!("race {race} kind {kind:#x} objects {objects:x?} away {away:?}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x621480), race);
                game_write(rva(0x63D9E4), away.0);
                game_write(rva(0x63D9E8), away.1);
                (f as *mut u8).write(kind);
                ((f + 1) as *mut u8).write(objects.0);
                ((f + 0x20) as *mut u8).write(kind | 1);
                ((f + 0x21) as *mut u8).write(objects.1);
            },
            &regions,
            copy.addr(),
            Regs {
                esi: f,
                ..Regs::SENTINEL
            },
            &mut |r| unsafe {
                draw_face_sides(r, &mut |p, r| {
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
