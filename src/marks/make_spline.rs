//! `marks::make_spline` (0x4B9A32): the next record of each of the four spline tables made
//! for object eax.
//!
//! In:  eax the object
//! Out: as the last steps leave them, ebp the first table's record
//!
//! See `marks/splines.rs`. One caller, `sub_4B96CD` (each of the settings' objects). The
//! second and third tables get the first table's record 0's centre and angle, whatever the
//! place (section 16). Refuses a negative place, an object past 0xFF or faces past
//! `g_faces`, an angle `fsincos` cannot take, before writing anything but the object.
//! Self-test: the original and the Rust version on the scene of `marks::measure_lines`
//! (places 0, 3, 7 and 8, kinds and angles by lot), the tables filled by lot; everything
//! put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::lines::{TEST_OBJECTS, TestScene};
use crate::marks::splines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::make_spline", rva(0x4B9A32), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let mut r = table_regions();
    r.extend(work_regions());
    r.extend([at(OBJECT_RVA), at(PLACE_RVA), at(OFFSET_RVA)]);
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { make_spline(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let scene = TestScene::new();
    let regions = regions();
    let inputs: Vec<Region> = vec![
        (crate::framework::game_addr(KINDS_RVA) as u32, 16),
        (crate::framework::game_addr(ANGLES_RVA) as u32, 48),
        (crate::framework::game_addr(SHIFT_RVA) as u32, 12),
    ];
    let saved = (capture(&regions), capture(&inputs));
    for place in [0u32, 3, 7, 8] {
        for (n, object) in TEST_OBJECTS.into_iter().enumerate() {
            let seed = place * 2 + n as u32 + 1;
            check_state(
                &mut t,
                &format!("place {place}, object {object:#x}"),
                call_original,
                Regs {
                    eax: object,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    scene.fill(seed, 1);
                    let mut s = seed;
                    for &(at, len) in &regions[..4] {
                        for k in 0..len as u32 / 4 {
                            s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
                            ((at + 4 * k) as *mut u32).write_unaligned(s >> 4);
                        }
                    }
                    game_write(KINDS_RVA, [seed % 4, (seed + 1) % 4, (seed + 2) % 4, 1]);
                    let mut v = [0u32; 18];
                    crate::scene::matrix3::test_floats(seed, &mut v);
                    game_write(ANGLES_RVA, <[u32; 12]>::try_from(&v[..12]).unwrap());
                    game_write(UP_RVA, <[u32; 6]>::try_from(&v[12..]).unwrap());
                    game_write(SHIFT_RVA, [s % 0x400, s % 0x300, s % 0x200]);
                    game_write(PLACE_RVA, place);
                },
                &regions,
                &rust,
            );
        }
    }
    unsafe { game_write(PLACE_RVA, u32::MAX) };
    t.check(
        unsafe {
            make_spline(&Regs {
                eax: TEST_OBJECTS[0],
                ..Regs::SENTINEL
            })
        }
        .is_err(),
        || "place -1: the Rust version went on".into(),
    );
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    drop(scene);
    t.finish()
}
