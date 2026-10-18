//! `race::cast_car_shadow` (0x4575C1): the work car's shadow (see `race/car_shadow.rs`):
//! with three corners or more its points, order, hull and polygon, the hull's list, and
//! the shadow on the faces round the car (`race::shadow_faces`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (its caller calls it inside `pusha`)
//!
//! Refuses what its parts refuse. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose faces call goes to a stand-in (the rest real), against the
//! Rust version with the same; the work car's corners set; all put back (skipped without
//! a race).

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva, shadow,
};
use crate::race::car_shadow::*;
use crate::race::shadow_points::{car_shadow_inputs, points_regions, set_car_shadow_inputs};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::cast_car_shadow", rva(0x4575C1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The shadow on the faces, in the game.
pub fn faces_in_game(r: &Regs) -> Result<Regs, crate::errors::Refused> {
    unsafe { shadow_faces(r, &mut |e| shadow_on_face(e, &mut face_part).map(|_| ())) }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { cast_car_shadow(r, &mut faces_in_game) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

static CALLS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn stand_in(_: u32, _: &mut Regs) {
    CALLS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

fn self_test() -> (usize, usize) {
    use std::sync::atomic::Ordering;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The ground under the car is searched in the stage's records.
    if !crate::physics::move_slot::in_race() {
        return t.skip("not in a race (no stage's records)");
    }
    if unsafe { game_read::<u32>(rva(0x621480)) } == 0 {
        return t.skip("no race loaded");
    }
    crate::install::stand_ins::set_handler(stand_in);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4575C1,
            len: 0x57,
            entry: &[0x8B, 0x2D, 0x2C, 0x19, 0x64, 0x00, 0x83, 0xFD],
            calls: &[0x0B, 0x10, 0x15, 0x23, 0x51],
            jumps_out: &[],
        },
        &|c| (c == rva(0x457618)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let mut regions = points_regions();
    regions.extend([
        g(0x620460, 8),
        at(HULL_COUNT_RVA),
        g(0x975510, 0x44 * 0x40),
        g(0x64199C, 0x104),
        at(rva(0x641930)),
    ]);
    let points: u32 = unsafe { game_read(rva(0x5CCCC0)) };
    let scene: u32 = unsafe { game_read(rva(0x5CCD20)) };
    regions.push((
        points.wrapping_add(scene.wrapping_mul(12)) + 0xC0,
        12 * 0x40,
    ));
    let mut kept = regions.clone();
    kept.extend(car_shadow_inputs());
    let saved = capture(&kept);
    for (seed, count) in [(1u32, 8u32), (0x99, 2), (0x4321, 8)] {
        let prepare = || {
            restore(&kept, &saved);
            set_car_shadow_inputs(seed, count);
            CALLS.store(0, Ordering::SeqCst);
        };
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (CALLS.load(Ordering::SeqCst), capture(&regions));
        prepare();
        let done = unsafe {
            cast_car_shadow(&Regs::SENTINEL, &mut |r| {
                CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(*r)
            })
        };
        let mine = (CALLS.load(Ordering::SeqCst), capture(&regions));
        t.check(done.is_ok() && theirs == mine, || {
            let diff = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
            format!(
                "seed {seed:#x} corners {count}: calls {} / {}; memory first differs at {diff:x?}",
                theirs.0, mine.0
            )
        });
    }
    restore(&kept, &saved);
    t.finish()
}
