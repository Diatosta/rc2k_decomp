//! `race::draw_tinted_face` (0x44F9F3): face esi drawn (`race::draw_course_face`) with
//! `g_polygonColour` times its object's tint (`dword_60A880`), put back after; see
//! `race/course_faces.rs`.
//!
//! In:  esi = the face
//! Out: the registers as the drawing leaves them (eax the tinted colour before it)
//!
//! Caller: `race::draw_list_entry` (place 2). Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose drawing call goes to a stand-in that notes the colour,
//! against the Rust version with the same: tints 0 and not, MMX on and off.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::course_faces::*;
use crate::race::face_stage::compare_plain;
use crate::race::level_init::note;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_tinted_face", rva(0x44F9F3), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { draw_tinted_face(r, &mut tinted_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(seen);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x44F9F3,
            len: 0xC3,
            entry: &[0xFF, 0x35, 0xCC, 0x40, 0x62, 0x00, 0x0F, 0xB6],
            calls: &[0xB7],
            jumps_out: &[],
        },
        &|c| {
            [(0x4537E3, 1)]
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
    let regions: Vec<Region> = vec![(game_addr(rva(0x6240CC)) as u32, 4)];
    let object = 0xFEu32;
    let mut kept = regions.clone();
    kept.extend([
        (game_addr(rva(0x5183A8)) as u32, 4),
        (game_addr(rva(0x60A880)) as u32 + 4 * object, 4),
    ]);
    let saved = capture(&kept);
    let mut face = vec![0u8; 0x40];
    face[1] = object as u8;
    let f = face.as_mut_ptr() as u32;
    for (tint, colour, mmx) in [
        (0u32, 0x8040_2010u32, 1u32),
        (0x80FF_4020, 0xC0C0_C0C0, 1),
        (0x80FF_4020, 0xC0C0_C0C0, 0),
        (0xFFFF_FFFF, 0x0102_0304, 0),
    ] {
        let line = compare_plain(
            &format!("tint {tint:#x} colour {colour:#x} mmx {mmx}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x5183A8), mmx);
                game_write(rva(0x6240CC), colour);
                game_write(rva(0x60A880) + 4 * object as usize, tint);
            },
            &regions,
            copy.addr(),
            Regs {
                esi: f,
                ..Regs::SENTINEL
            },
            &mut |r| unsafe {
                draw_tinted_face(r, &mut |p, r| {
                    seen(p, r);
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

/// Notes the place and the colour at the call (in eax's slot of the note).
fn seen(place: u32, r: &mut Regs) {
    let mut noted = Regs {
        eax: unsafe { crate::framework::game_read(rva(0x6240CC)) },
        ..*r
    };
    note(place, &mut noted);
}
