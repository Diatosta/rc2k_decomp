//! `race::scroll_map` (0x45B527): the work copy's car on the course map, its picture's
//! corner moved and redrawn when the car nears an edge, its place in the picture; see
//! `race/map_scroll.rs`.
//!
//! In:  nothing (the work copy `g_car`)
//! Out: the registers as it leaves them (its caller goes on with a call)
//!
//! Refuses (before anything) no course faces or points. Callers: `race::scroll_maps`
//! (three). Shadow mode runs the original alone (the redraw writes a picture). Self-test: a
//! copy whose redraw goes to a stand-in, against the Rust version with the same: a test
//! face and point, places inside and outside the picture, fewer than two checkpoints.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::level_init::note;
use crate::race::map_scroll::{scroll_map, scroll_part};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::scroll_map", rva(0x45B527), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = scroll_part(1, r) {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    scroll_part(1, r)?;
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
            address: 0x45B527,
            len: 0x13C,
            entry: &[0xC7, 0x05, 0xB4, 0xAA, 0x71, 0x00, 0x00, 0x00],
            calls: &[0xF4],
            jumps_out: &[],
        },
        &|c| (c == rva(0x45B663)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let mut faces = vec![0u32; 0x20];
    faces[0x10 + 1] = 2;
    let points: Vec<u32> = vec![0, 0, 0, 0, 0, 0, 0x12_0000, 0x500, 0x34_0000];
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let regions: Vec<Region> = vec![g(0x71AAAC, 0x24)];
    let mut kept = regions.clone();
    kept.extend([
        at(rva(0x623640)),
        at(rva(0x5CCCBC)),
        at(rva(0x5CCCC0)),
        at(rva(0x71A070)),
        at(rva(0x71A10C)),
        at(rva(0x71A114)),
        at(rva(0x9E3738 + 4 * 3)),
        at(rva(0x9FC940 + 4 * 3)),
        at(rva(0xA019A8 + 4 * 3)),
    ]);
    let saved = capture(&kept);
    let f = faces.as_mut_ptr() as u32;
    for (count, place, corner) in [
        (5u32, (0x12_0000u32, 0x34_0000u32), (0u32, 0u32)),
        (5, (0x15_0000, 0x30_0000), (0x10, 0x20)),
        (5, (0x100_0000, 0x10_0000), (0x40, 0x40)),
        (1, (0, 0), (0, 0)),
    ] {
        let line = compare_plain(
            &format!("checkpoints {count} place {place:x?} corner {corner:x?}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x623640), count);
                game_write(rva(0x5CCCBC), f);
                game_write(rva(0x5CCCC0), points.as_ptr() as u32);
                game_write(rva(0x71A070), 3u32);
                game_write(rva(0x9E3738 + 4 * 3), 1u32);
                game_write(rva(0x9FC940 + 4 * 3), 0x50_0000u32);
                game_write(rva(0xA019A8 + 4 * 3), 0x60_0000u32);
                game_write(rva(0x71A10C), place.0);
                game_write(rva(0x71A114), place.1);
                game_write(rva(0x71AAAC), corner.0);
                game_write(rva(0x71AAB0), corner.1);
            },
            &regions,
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                scroll_map(r, &mut |p, r| {
                    note(p, r);
                    Ok(())
                })
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&kept, &saved);
    drop(faces);
    t.finish()
}
