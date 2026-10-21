//! `scene::widen_object_radius` (0x426A7F): the stage objects' radius (`dword_5CCE14`)
//! raised to a point's distance from the middle when that is larger.
//!
//! In:  eax and ecx the point (whole numbers)
//! Out: eax the distance (`fistp` of the root of eax^2 + ecx^2, through `dword_62400C`);
//!      everything else preserved
//! The radius takes it when distance - radius is not negative (`js`). One caller,
//! 0x42687C (`scene::load_stage_objects`'s point loop).
//! Shadow mode: both, compared.
//! Self-test: the original against the Rust version on points of the test's (the two
//! variables put back).
//!
//! Converted 2026-10-21: the Rust version is `widen_object_radius_plain`, through
//! `rc2k_core::scene::distance::{radius_distance, widened}` (an `f64` root; the x87
//! version, with no Rust callers, went). Shadow mode and the self-test hold the whole
//! numbers to one either way at extended precision, exact at double.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::floats::{
    Floats, ROUNDED, Tolerance, at_test_precisions, check_state_floats, dispatch_state_floats,
};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::widen_object_radius",
    rva(0x426A7F),
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

/// dword_62400C the scratch, dword_5CCE14 the radius.
const SCRATCH_RVA: usize = rva(0x62400C);
const RADIUS_RVA: usize = rva(0x5CCE14);

// ---- Rust version ----

// Converted: `rc2k_core::scene::distance::{radius_distance, widened}`
pub fn widen_object_radius_plain(x: u32, z: u32) -> u32 {
    use rc2k_core::scene::distance::{radius_distance, widened};
    let d = radius_distance(x as i32, z as i32) as u32;
    unsafe {
        game_write(SCRATCH_RVA, d);
        if let Some(radius) = widened(d, game_read(RADIUS_RVA)) {
            game_write(RADIUS_RVA, radius);
        }
    }
    d
}

// ---- hook ----

fn regions() -> [Region; 2] {
    [at(SCRATCH_RVA), at(RADIUS_RVA)]
}

/// The whole numbers made from the root: within one at extended precision.
fn float_places() -> [(Region, Tolerance); 2] {
    regions().map(|r| (r, ROUNDED))
}

const FLOAT_REGS: [(u8, Tolerance); 1] = [(EAX, ROUNDED)];

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, widen_object_radius_plain(r.eax, r.ecx)))
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = float_places();
    let floats = Floats {
        memory: &places,
        regs: &FLOAT_REGS,
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = regions();
    let saved = capture(&all);
    let places = float_places();
    let tolerant = Floats {
        memory: &places,
        regs: &FLOAT_REGS,
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for (x, z) in [
            (0u32, 0u32),
            (3, 4),
            (0xFFFF_FFFD, 4),
            (1000, 1),
            (0x7FFF, 0x7FFF),
            (5, 5),
        ] {
            for radius in [0u32, 5, 7, 0x8000_0000] {
                check_state_floats(
                    &mut t,
                    "scene::widen_object_radius",
                    &format!("control word {word:#x}, point {x:#x} {z:#x}, radius {radius:#x}"),
                    call_original,
                    Regs {
                        eax: x,
                        ecx: z,
                        ..Regs::SENTINEL
                    },
                    &|| unsafe { game_write(RADIUS_RVA, radius) },
                    &all,
                    held,
                    &rust,
                );
            }
        }
    });
    restore(&all, &saved);
    t.finish()
}
