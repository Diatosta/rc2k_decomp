//! `race::draw_back_face` (0x452B58): a two-sided face's back: the corners turned over
//! (and the first texture's order by `byte_63DA7C`), the back's colours and fog bytes,
//! drawn (`race::draw_face`), turned back; see `race/course_faces.rs`.
//!
//! In:  esi = the face
//! Out: the registers as it leaves them (callers return)
//!
//! Callers: `sub_452393` (two). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose `draw_face` call goes to a stand-in, against the Rust version
//! with the same: test faces, two-sided or not, triangles and quads, each order.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::course_faces::*;
use crate::race::face_stage::compare_plain;
use crate::race::level_init::note;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_back_face", rva(0x452B58), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { draw_back_face(r, &mut |_, r| draw_face(r, &mut face_part)) } {
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
            address: 0x452B58,
            len: 0x117,
            entry: &[0x83, 0x3D, 0x34, 0xDA, 0x63, 0x00, 0x00, 0x0F],
            calls: &[0xC1],
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
        (game_addr(rva(0x63D9EC)) as u32, 0x10),
        (game_addr(rva(0x63DA30)) as u32, 0x44),
    ];
    let saved = capture(&regions);
    let mut face = vec![0u8; 0x40];
    for (k, b) in face.iter_mut().enumerate() {
        *b = (k as u8).wrapping_mul(53).wrapping_add(7);
    }
    let f = face.as_mut_ptr() as u32;
    for (two, fourth, order) in [
        (8u32, 4u32, 0u32),
        (8, u32::MAX, 5),
        (0, 4, 2),
        (8, 4, 0xE),
        (8, 4, 7),
    ] {
        let line = compare_plain(
            &format!("two-sided {two} fourth {fourth:#x} order {order}"),
            &|| unsafe {
                restore(&regions, &saved);
                game_write(rva(0x63DA34), two);
                game_write(rva(0x63DA30), order);
                for (n, v) in [1u32, 2, 3, fourth].iter().enumerate() {
                    game_write(rva(0x63D9EC) + 4 * n, *v);
                }
            },
            &regions,
            copy.addr(),
            Regs {
                esi: f,
                ..Regs::SENTINEL
            },
            &mut |r| unsafe {
                draw_back_face(r, &mut |p, r| {
                    note(p, r);
                    Ok(())
                })
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&regions, &saved);
    drop(face);
    t.finish()
}
