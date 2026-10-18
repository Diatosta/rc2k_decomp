//! `race::add_depth_entry` (0x454D2A): `race::face_depth` of face ebx, then entry
//! `dword_63DC4C` of `dword_9D3730` (0x1000 of 8 bytes): +0 the key, +4 ebx; the count + 1.
//!
//! In:  ebx = the face's number
//! Out: edx the entry, ecx the key, ebx & 0x7FFFFFFF, CF clear (`clc`; its callers in
//!      `sub_454C99` jump on: not read)
//!
//! See `race/frame_draw.rs`. It begins with its call: private copy (0x2D, the call
//! relocated), manual patch. Refuses (before anything) an entry past the 0x1000, or what
//! `face_depth` refuses. Shadow mode: by state. Self-test: a test face (`g_courseFaces`
//! pointed at it) on the game's points, counts 0, 7, 0xFFF; all put back (skipped without
//! points); 0x1000 refused.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::face_depth::depth_regions;
use crate::race::frame_draw::*;
use crate::scene::project_point::{
    POINT_COUNT_RVA, POINTS_RVA, RECORDS_RVA, regions as point_regions,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::add_depth_entry", rva(0x454D2A), thunk, &ORIGINAL)
    .private_copy(0x2D)
    .relocate_calls(&[0x00])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FACES: usize = rva(0x5CCCBC); // g_courseFaces

fn regions(ebx: u32) -> Vec<Region> {
    let mut v = depth_regions(ebx);
    v.push(at(DEPTH_COUNT_RVA));
    let n: u32 = unsafe { game_read(DEPTH_COUNT_RVA) };
    if n < 0x1000 {
        v.push((game_addr(DEPTH_ENTRIES_RVA) as u32 + 8 * n, 8));
    }
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { add_depth_entry(r.ebx) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(r.ebx), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let there = unsafe {
        game_read::<u32>(RECORDS_RVA) != 0
            && game_read::<u32>(POINTS_RVA) != 0
            && game_read::<u32>(POINT_COUNT_RVA) >= 4
    };
    if !there {
        return t.skip("no scene points (no race loaded)");
    }
    let mut faces = vec![0u8; 0x80];
    let base = faces.as_mut_ptr() as u32;
    let entries = game_addr(DEPTH_ENTRIES_RVA) as u32;
    let mut kept: Vec<Region> = vec![at(FACES), at(DEPTH_COUNT_RVA), (entries, 0x8000)];
    for p in 0..4 {
        kept.extend(point_regions(p));
    }
    kept.extend([at(FACE_KEY_RVA), at(rva(0x63DC44))]);
    let saved = capture(&kept);
    unsafe {
        let face = base + 0x40;
        (face as *mut u8).write(0x12);
        for k in 0..4u32 {
            ((face + 4 + 4 * k) as *mut u32).write_unaligned(k);
        }
    }
    for (count, ebx_high) in [(0u32, 0u32), (7, 0x8000_0000), (0xFFF, 0)] {
        let ebx = 1 | ebx_high;
        unsafe {
            game_write(FACES, base.wrapping_sub(ebx_high << 6));
            game_write(DEPTH_COUNT_RVA, count);
        }
        let regions = regions(ebx);
        check_state(
            &mut t,
            &format!("count {count:#x} ebx {ebx:#x}"),
            call_original,
            Regs {
                ebx,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&kept, &saved);
                game_write(FACES, base.wrapping_sub(ebx_high << 6));
                game_write(DEPTH_COUNT_RVA, count);
                std::ptr::write_bytes((entries + 8 * count) as *mut u8, 0xAA, 8);
            },
            &regions,
            &rust,
        );
    }
    let refused = unsafe {
        game_write(DEPTH_COUNT_RVA, 0x1000u32);
        add_depth_entry(1).is_err()
    };
    restore(&kept, &saved);
    t.check(refused, || "entry 0x1000 not refused".into());
    drop(faces);
    t.finish()
}
