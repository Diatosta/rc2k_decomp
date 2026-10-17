//! `marks::make_shape_set` (0x4BD1A1): the next shape set made of object ebp's faces of the
//! two kinds.
//!
//! In:  ebp the object
//! Out: edi the set, the rest as `marks::find_farthest_pairs` leaves them
//!
//! See `marks/shape_points.rs`. One caller, `marks::make_shape_sets`. Refuses a set count
//! past 0x22, an object past 0xFF or faces past `g_faces`, a tilt `fsincos` cannot take,
//! before writing. Self-test: the original and the Rust version on the scene of
//! `marks::measure_lines` (faces of both kinds), 0, 1 and 0x22 sets made, tilts by lot,
//! the refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::lines::{TEST_OBJECTS, TestScene};
use crate::marks::shape_points::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::make_shape_set", rva(0x4BD1A1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { make_shape_set(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = shape_set_regions();
    let made: u32 = unsafe { crate::framework::game_read(SET_COUNT_RVA) };
    if made < SETS {
        regions.push((
            crate::framework::game_addr(SET_RVA) as u32 + made * SET_LEN,
            SET_LEN as usize,
        ));
    }
    regions.push(crate::service::harness::at(SET_COUNT_RVA));
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let scene = TestScene::new();
    let regions = shape_test_regions();
    let saved = capture(&regions);
    for made in [0u32, 1, SETS] {
        for (n, object) in TEST_OBJECTS.into_iter().enumerate() {
            let seed = made * 2 + n as u32 + 1;
            check_state(
                &mut t,
                &format!("{made:#x} made, object {object:#x}"),
                call_original,
                Regs {
                    ebp: object,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    restore(&regions, &saved);
                    scene.fill(seed, 1);
                    shape_test_faces(&scene, seed);
                    shape_test_inputs(seed);
                    game_write(SET_COUNT_RVA, made);
                },
                &regions,
                &rust,
            );
        }
    }
    unsafe { game_write(SET_COUNT_RVA, SETS + 1) };
    t.check(
        unsafe {
            make_shape_set(&Regs {
                ebp: TEST_OBJECTS[0],
                ..Regs::SENTINEL
            })
        }
        .is_err(),
        || "0x23 sets made: the Rust version went on".into(),
    );
    restore(&regions, &saved);
    drop(scene);
    t.finish()
}
