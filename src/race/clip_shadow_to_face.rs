//! `race::clip_shadow_to_face` (0x457960): the shadow's polygon cut to one course face
//! (see `race/car_shadow.rs`: `sub_457A85` per edge, `sub_457BE8` the side test and
//! `sub_457CA8` the crossing, covered here; the x87 the original's instructions).
//!
//! In:  nothing (the face's corners `dword_641938` ..)
//! Out: the registers as it leaves them (its caller calls it inside `pusha`)
//!
//! Shadow mode: by state. Self-test: scene points of the test's (`g_scenePoints` pointed
//! at them), a square cut by triangles inside, across and away, the work car near and far;
//! all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::car_shadow::*;
use crate::race::frame_draw::all_regs;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::clip_shadow_to_face", rva(0x457960), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// What cutting writes (the scene's new points are the caller's to give).
pub fn clip_regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        g(0x975510, 0x44 * 0x40),
        g(0x977710, 0x200),
        at(rva(0x6240F8)),
        at(rva(0x624114)),
        g(0x641948, 0x4C),
        g(0x639810, 4),
        g(0x639824, 0xC),
        at(rva(0x62400C)),
        g(0x623F88, 0x28),
        at(rva(0x62419C)),
        at(rva(0x6241AC)),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { clip_shadow_to_face(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = clip_regions();
    regions.push(shadow_points_region());
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The scene points the cutting may add (from `dword_641990`).
pub fn shadow_points_region() -> Region {
    unsafe {
        let from: u32 = crate::framework::game_read(rva(0x641990));
        let points: u32 = crate::framework::game_read(rva(0x5CCCC0));
        (points.wrapping_add(from.wrapping_mul(12)), 12 * 0x20)
    }
}

/// A test square of four entries (points 0 .. 3) and a face's three corners (4 .. 6),
/// the car at (cx, cz); `points` the test's scene points.
pub fn set_square(points: &mut [u32], face: [(i32, i32); 3], car: (u32, u32)) {
    let square = [
        (0x1000, 0x1000),
        (0x9000, 0x1000),
        (0x9000, 0x9000),
        (0x1000, 0x9000),
    ];
    for (k, &(x, z)) in square.iter().chain(face.iter()).enumerate() {
        points[3 * k] = x as u32;
        points[3 * k + 1] = 0;
        points[3 * k + 2] = z as u32;
    }
    unsafe {
        game_write(rva(0x5CCCC0), points.as_ptr() as u32);
        let entries = game_addr(rva(0x975510)) as u32;
        let list = game_addr(rva(0x977710)) as u32;
        for k in 0..4u32 {
            let e = entries + 0x44 * (k + 1);
            (e as *mut u32).write_unaligned(k);
            ((e + 0x10) as *mut u32).write_unaligned((k as f32 * 0.25).to_bits());
            ((e + 0x14) as *mut u32).write_unaligned((1.0 - k as f32 * 0.25).to_bits());
            ((e + 0x20) as *mut u8).write(0x40 * k as u8);
            ((e + 0x21) as *mut u8).write(0xFF - 0x30 * k as u8);
            ((list + 4 * k) as *mut u32).write_unaligned(k + 1);
        }
        game_write(rva(0x6240F8), 4u32);
        game_write(rva(0x624114), 0x20u32);
        game_write(rva(0x641990), 0x10u32);
        game_write(rva(0x641938), 4u32);
        game_write(rva(0x64193C), 5u32);
        game_write(rva(0x641940), 6u32);
        game_write(rva(0x71A10C), car.0);
        game_write(rva(0x71A114), car.1);
    }
}

pub const FACES: [[(i32, i32); 3]; 4] = [
    [(0, 0), (0x20000, 0), (0, 0x20000)],
    [(0x5000, 0), (0x5000, 0x20000), (0x20000, 0x5000)],
    [(0x40000, 0x40000), (0x50000, 0x40000), (0x40000, 0x50000)],
    [(0x2000, 0x2000), (0x8000, 0x3000), (0x4000, 0x8000)],
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut points = vec![0u32; 3 * 0x40];
    let regions = clip_regions();
    let mut kept = regions.clone();
    kept.extend([
        at(rva(0x5CCCC0)),
        at(rva(0x641990)),
        (game_addr(rva(0x641938)) as u32, 0xC),
        (game_addr(rva(0x71A10C)) as u32, 0xC),
    ]);
    let saved = capture(&kept);
    let p = points.as_mut_ptr() as u32;
    for (n, face) in FACES.iter().enumerate() {
        for car in [(0x5000u32, 0x5000u32), (0x50_0000, 0x5000)] {
            let mut all = regions.clone();
            all.push((p, 12 * 0x40));
            check_state(
                &mut t,
                &format!("face {n} car {car:x?}"),
                call_original,
                Regs::SENTINEL,
                &|| {
                    restore(&kept, &saved);
                    let points = unsafe { std::slice::from_raw_parts_mut(p as *mut u32, 3 * 0x40) };
                    set_square(points, *face, car);
                },
                &all,
                &rust,
            );
        }
    }
    restore(&kept, &saved);
    drop(points);
    t.finish()
}
