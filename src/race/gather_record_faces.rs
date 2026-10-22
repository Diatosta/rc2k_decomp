//! `race::gather_record_faces` (0x454C99): the faces of one course record into the draw
//! lists (see `race/frame_draw.rs`).
//!
//! In:  esi = the course record (0x20 bytes: +8 first face, +0xC last, -1 none)
//! Out: the registers as it leaves them (ebx the face after the last; eax, ecx, edx as the
//!      last face's tests and entry left them); its callers call it inside `pusha`
//!
//! Each face, `dword_5CCCD0` the face: kinds 0xFF and below 0x10 that `race::face_hidden`
//! lets through go to `race::add_depth_entry` (the object's flag in `byte_63DD5C` /
//! `byte_63DC5C`) or `race::add_sort_entry` (the object's kind byte, `byte_63DF5C` /
//! `byte_63DE5C`); for kinds below 0x10 the object is +1, or +0x21 when +1 is 0. Refuses
//! (before anything) a last face of -1, no points, or more faces than either list has room
//! for. Shadow mode: by state (the lists, their counts, the tests' dwords, the faces'
//! point records). Self-test: test records over the game's faces 0 .. 39 (ranges, one face,
//! none, first above last), the lists' counts set; all put back (skipped without a stage).
//! Caller: `race::gather_visible_faces`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::gather_record_faces", rva(0x454C99), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { gather_record_faces(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &gather_regions(&[r.esi]), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// Whether a stage is there to gather from: faces, point records and points.
pub fn stage_there() -> bool {
    unsafe {
        game_read::<u32>(rva(0x5CCCBC)) != 0
            && game_read::<u32>(crate::scene::project_point::RECORDS_RVA) != 0
            && game_read::<u32>(crate::scene::project_point::POINTS_RVA) != 0
            && game_read::<u32>(rva(0x5CCD1C)) >= 40 // g_courseFaceCount
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !stage_there() {
        return t.skip("no stage loaded");
    }
    let mut record = vec![0u8; 0x20];
    let at = record.as_mut_ptr() as u32;
    for (first, last, counts) in [
        (0u32, 9u32, 0x10u32),
        (10, 39, 0),
        (5, 5, 0x100),
        (u32::MAX, 3, 0),
        (8, 7, 0),
    ] {
        unsafe {
            ((at + 8) as *mut u32).write_unaligned(first);
            ((at + 0xC) as *mut u32).write_unaligned(last);
        }
        let regions: Vec<Region> = gather_regions(&[at]);
        let saved = capture(&regions);
        check_state(
            &mut t,
            &format!("faces {first:#x} .. {last:#x} counts {counts:#x}"),
            call_original,
            Regs {
                esi: at,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&regions, &saved);
                game_write(SORT_COUNT_RVA, counts);
                game_write(DEPTH_COUNT_RVA, counts);
            },
            &regions,
            &rust,
        );
        restore(&regions, &saved);
    }
    drop(record);
    t.finish()
}
