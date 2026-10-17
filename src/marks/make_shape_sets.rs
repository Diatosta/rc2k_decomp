//! `marks::make_shape_sets` (0x4BD019): a shape set made for each of the settings' objects.
//!
//! In:  nothing
//! Out: esi the settings, the rest as the last set leaves them (eax the kind with none)
//!
//! See `marks/shape_points.rs`. One caller, `marks::make_masks`. Refuses a kind of 0x22 or
//! more, more than 19 objects, what `make_shape_set` refuses for any of them, before
//! writing. Self-test: the original and the Rust version on the scene of
//! `marks::measure_lines`, 0, 1 and 3 objects, the refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::lines::{TEST_OBJECTS, TestScene};
use crate::marks::shape_points::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::make_shape_sets", rva(0x4BD019), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { make_shape_sets(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = shape_sets_regions();
    regions.push((
        crate::framework::game_addr(SET_RVA) as u32,
        19 * SET_LEN as usize,
    ));
    regions.push(at(SET_COUNT_RVA));
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let scene = TestScene::new();
    let settings = scene.settings.as_ptr() as u32;
    let regions = shape_test_regions();
    let saved = capture(&regions);
    for objects in [0u32, 1, 3] {
        for seed in [1u32, 2] {
            check_state(
                &mut t,
                &format!("{objects} objects, seed {seed}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    restore(&regions, &saved);
                    scene.fill(seed + objects, 1);
                    shape_test_faces(&scene, seed);
                    let at = |o: u32| (settings + o) as *mut u32;
                    at(0x26B8).write_unaligned(objects);
                    for k in 0..objects {
                        at(0x26BC + 4 * k).write_unaligned(TEST_OBJECTS[k as usize % 2]);
                    }
                    game_write(crate::marks::kind_values::KIND_RVA, seed);
                },
                &regions,
                &rust,
            );
        }
    }
    unsafe { game_write(crate::marks::kind_values::KIND_RVA, SETS) };
    t.check(unsafe { make_shape_sets(&Regs::SENTINEL) }.is_err(), || {
        "kind 0x22: the Rust version went on".into()
    });
    restore(&regions, &saved);
    drop(scene);
    t.finish()
}
