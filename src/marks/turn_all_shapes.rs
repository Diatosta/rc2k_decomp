//! `marks::turn_all_shapes` (0x4BD122): for each of the settings' objects the shape set
//! turned by the two shapes' eased steps.
//!
//! In:  nothing
//! Out: esi the last object's offset (the settings with none), the rest as the last turn
//!
//! See `marks/shape_points.rs`. One caller, 0x4BDF87. The object's offset is not used: the
//! same set is turned each time (section 16). Refuses what `marks::turn_shape_points` would,
//! before writing. Self-test: the original and the Rust version with 0, 1 and 3 objects,
//! steps, switches and sets by lot, the refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::lines::TestScene;
use crate::marks::shape_points::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::turn_all_shapes", rva(0x4BD122), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn regions() -> Vec<Region> {
    let mut r = turn_regions();
    r.extend(unsafe { target_regions() });
    r.extend(crate::marks::turns::step_regions());
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { turn_all_shapes(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// The steps, ends and switches the test sets (as `turn_all_shapes` reads them).
const INPUTS: [usize; 8] = [
    rva(0x82FC56),
    rva(0x89C974),
    rva(0x884FC4),
    rva(0x844604),
    rva(0x82FC7A),
    rva(0x89C97C),
    rva(0x884FC8),
    rva(0x844608),
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let scene = TestScene::new();
    let settings = scene.settings.as_ptr() as u32;
    let mut inputs: Vec<Region> = INPUTS.iter().map(|&r| at(r)).collect();
    inputs.extend(set_inputs());
    let mut regions = turn_regions();
    regions.extend(crate::marks::turns::step_regions());
    let saved = (capture(&inputs), capture(&regions));
    let targets = vec![0xAAu8; 12 * 2 * MOST_POINTS as usize];
    let tp = targets.as_ptr() as u32;
    regions.push((tp, targets.len()));
    for objects in [0u32, 1, 3] {
        for seed in [1u32, 2, 3] {
            check_state(
                &mut t,
                &format!("{objects} objects, seed {seed}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    std::ptr::write_bytes(tp as *mut u8, 0xAA, targets.len());
                    ((settings + 0x26B8) as *mut u32).write_unaligned(objects);
                    fill_set(seed * 5 + objects, [seed * 3, 7], tp);
                    let mut v = [0u32; 4];
                    crate::scene::matrix3::test_floats(seed + 60, &mut v);
                    let values = [
                        seed * 9 % 0x80,
                        v[0],
                        v[1],
                        seed % 2,
                        seed * 5 % 0x80,
                        v[2],
                        v[3],
                        0,
                    ];
                    for (r, x) in INPUTS.iter().zip(values) {
                        game_write(*r, x);
                    }
                },
                &regions,
                &rust,
            );
        }
    }
    unsafe {
        ((settings + 0x26B8) as *mut u32).write_unaligned(1);
        fill_set(3, [MOST_POINTS + 1, 0], tp);
    }
    t.check(unsafe { turn_all_shapes(&Regs::SENTINEL) }.is_err(), || {
        "0x41 points: the Rust version went on".into()
    });
    regions.pop();
    restore(&inputs, &saved.0);
    restore(&regions, &saved.1);
    drop(scene);
    t.finish()
}
