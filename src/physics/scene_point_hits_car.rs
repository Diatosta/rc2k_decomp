//! `physics::scene_point_hits_car` (0x45E7BF): `physics::point_hits_car` for the scene
//! point whose number is the dword at esi+4 (`g_scenePoints` + 12 n), with no offset:
//! `dword_614108` + 1, `dword_6D3B40` .. = 0.
//!
//! In:  esi the record (+4 the point's number), ebp the object
//! Out: CF as `point_hits_car` leaves it; ebx the number, esi the point; the rest kept
//!
//! Caller: `sub_45E3BF` (0x45E48C, `jb`).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) with test points (`g_scenePoints` pointed at them for the test), the car's
//! figures by lot, under three precisions.

use crate::errors::Refused;
use crate::framework::{EBX, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::flow::{a, fill3, g, p};
use crate::physics::point_hits_car::{self, point_hits_car};
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::scene_point_hits_car",
    rva(0x45E7BF),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `g_scenePoints`: the pointer to the scene's points (three whole numbers each).
pub const SCENE_POINTS: usize = 0x5CCCC0;

/// The Rust version: (inside, ebx, esi).
pub unsafe fn scene_point_hits_car(record: u32, object: u32) -> (bool, u32, u32) {
    let number = unsafe { ((record + 4) as *const u32).read_unaligned() };
    let point = number.wrapping_mul(12).wrapping_add(g(SCENE_POINTS));
    p(0x614108, g(0x614108).wrapping_add(1));
    fill3(0x6D3B40, 0);
    (unsafe { point_hits_car(point, object) }, number, point)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (inside, ebx, esi) = unsafe { scene_point_hits_car(r.esi, r.ebp) };
    Ok(Left::carry(inside).with(EBX, ebx).with(ESI, esi))
}

pub fn regions() -> Vec<Region> {
    let mut all = point_hits_car::regions();
    all.extend([(a(0x614108), 4), (a(0x6D3B40), 12)]);
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions();
    all.extend(point_hits_car::inputs());
    all.push((a(SCENE_POINTS), 4));
    let saved = capture(&all);
    let mut points = [0u32; 3 * 16];
    let mut record = [0xAAAA_AAAAu32; 2];
    let mut object = [0u32; 0x10];
    let (pts, rec, ob) = (
        points.as_mut_ptr() as u32,
        record.as_mut_ptr() as u32,
        object.as_mut_ptr() as u32,
    );
    all.extend([(pts, 0xC0), (rec, 8), (ob, 0x40)]);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..16u32 {
                let set = || {
                    point_hits_car::set_inputs(seed, [10u32, 1, 2, 6][(seed % 4) as usize]);
                    // The calls' counter it counts up: the same start for both sides.
                    p(0x614108, seed);
                    p(SCENE_POINTS, pts);
                    let mut f = [0u32; 0x33];
                    test_floats(seed * 11 + 3, &mut f);
                    for (k, v) in f.iter().enumerate().take(0x30) {
                        let v = (*v as i32).wrapping_rem(0x18000) as u32;
                        unsafe { ((pts + 4 * k as u32) as *mut u32).write(v) };
                    }
                    for k in 0..3 {
                        let o = (f[0x30 + k] as i32).wrapping_rem(0x8000) as u32;
                        unsafe { ((ob + 0x30 + 4 * k as u32) as *mut u32).write(o) };
                    }
                    unsafe { ((rec + 4) as *mut u32).write(seed % 16) };
                };
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}"),
                    call_original,
                    Regs {
                        esi: rec,
                        ebp: ob,
                        ..Regs::SENTINEL
                    },
                    &set,
                    &all,
                    &rust,
                );
            }
        });
    }
    all.truncate(all.len() - 3);
    restore(&all, &saved);
    t.finish()
}
