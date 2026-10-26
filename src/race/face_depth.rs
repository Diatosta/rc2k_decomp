//! `race::face_depth` (0x4552D8): a course face's depth key into `dword_63DC40`.
//!
//! In:  ebx = the face's number (records of 0x40 bytes at `g_courseFaces`)
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! A kind (+0) of 0x10 or more: its first point's depth (`fistp`; `dword_63DC44` its x);
//! else, with its object's (+1) record +0xE bit 1, the largest of its four points' depths
//! (each `fistp`, through `dword_62400C`); otherwise the four depths summed as floats,
//! `fistp`'d and shifted right 2 (arithmetic). Then + 0x2000000 and shifted right 10. The
//! points' scene records are made when not there (as `sub_44CF33`). See
//! `race/frame_draw.rs`. Caller: `race::add_depth_entry`. Refuses (before anything) no point
//! records or points. Shadow mode: by state (the two dwords, the four records,
//! `dword_62400C`). Self-test: test faces (`g_courseFaces` pointed at them) on the game's
//! points 0 .. 7, each kind of face, the object flag on and off, records cleared so they
//! are made; all put back (skipped without points).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::scene::project_point::{
    POINT_COUNT_RVA, POINTS_RVA, RECORDS_RVA, regions as point_regions,
};
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::face_depth", rva(0x4552D8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FACES: usize = rva(0x5CCCBC); // g_courseFaces

/// What it may write for face `ebx`.
pub fn depth_regions(ebx: u32) -> Vec<Region> {
    let mut v = vec![at(FACE_KEY_RVA), at(rva(0x63DC44))];
    let faces: u32 = unsafe { game_read(FACES) };
    if faces != 0 {
        let face = (ebx << 6).wrapping_add(faces);
        for k in 0..4 {
            let point = unsafe { (face.wrapping_add(4 + 4 * k) as *const u32).read_unaligned() };
            v.extend(point_regions(point));
        }
    }
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { face_depth(r.ebx) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &depth_regions(r.ebx), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let there = unsafe {
        game_read::<u32>(RECORDS_RVA) != 0
            && game_read::<u32>(POINTS_RVA) != 0
            && game_read::<u32>(POINT_COUNT_RVA) >= 8
    };
    if !there {
        return t.skip("no scene points (no race loaded)");
    }
    let mut faces = vec![0u8; 0x40 * 4];
    let base = faces.as_mut_ptr() as u32;
    // Objects 1 and 2: +0xE bit 1 off and on.
    let objects = game_addr(rva(0x979F50)) as u32;
    let mut kept: Vec<Region> = vec![
        at(FACES),
        (objects + 0x40 + 0xE, 1),
        (objects + 0x80 + 0xE, 1),
    ];
    for p in 0..8 {
        kept.extend(point_regions(p));
    }
    kept.extend([at(FACE_KEY_RVA), at(rva(0x63DC44))]);
    let saved = capture(&kept);
    for (n, kind, object, points, made) in [
        (0u32, 0x03u8, 1u8, [0u32, 1, 2, 3], true),
        (1, 0x05, 2, [4, 5, 6, 7], true),
        (2, 0x12, 1, [3, 0, 0, 0], false),
        (3, 0x0F, 2, [7, 6, 5, 4], false),
        (1, 0x00, 1, [1, 3, 5, 7], true),
    ] {
        unsafe {
            let face = base + 0x40 * n;
            for (k, &p) in points.iter().enumerate() {
                ((face + 4 + 4 * k as u32) as *mut u32).write_unaligned(p);
            }
            game_write(FACES, base);
        }
        let regions = depth_regions(n);
        check_state(
            &mut t,
            &format!("face {n} kind {kind:#x} object {object} points {points:?} made {made}"),
            call_original,
            Regs {
                ebx: n,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&kept, &saved);
                let face = base + 0x40 * n;
                (face as *mut u8).write(kind);
                ((face + 1) as *mut u8).write(object);
                for (k, &p) in points.iter().enumerate() {
                    ((face + 4 + 4 * k as u32) as *mut u32).write_unaligned(p);
                }
                ((objects + 0x40 + 0xE) as *mut u8).write(0);
                ((objects + 0x80 + 0xE) as *mut u8).write(2);
                game_write(FACES, base);
                if made {
                    for &p in &points {
                        let record = ((p & 0xFFF) << 5) + game_read::<u32>(RECORDS_RVA);
                        (record as *mut u32).write_unaligned(u32::MAX);
                    }
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    drop(faces);
    t.finish()
}
