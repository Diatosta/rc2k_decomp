//! `scene::widen_object_radius` (0x426A7F): the stage objects' radius (`dword_5CCE14`)
//! raised to a point's distance from the middle when that is larger.
//!
//! In:  eax and ecx the point (whole numbers)
//! Out: eax the distance (`fistp` of the root of eax^2 + ecx^2, through `dword_62400C`);
//!      everything else preserved
//! The radius takes it when distance - radius is not negative (`js`). One caller,
//! 0x42687C (`scene::load_stage_objects`'s point loop). The x87 code is the original's.
//! Shadow mode: both, compared.
//! Self-test: the original against the Rust version on points of the test's (the two
//! variables put back).

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::physics::flow::{base, x87_run};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
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

pub fn widen_object_radius(x: u32, z: u32) -> u32 {
    x87_run!(
        "mov dword ptr [{b} + 0x22400C], {x}",
        "fild dword ptr [{b} + 0x22400C]",
        ".byte 0xD8, 0xC8",
        "mov dword ptr [{b} + 0x22400C], {z}",
        "fild dword ptr [{b} + 0x22400C]",
        ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1",
        ".byte 0xD9, 0xFA",
        "fistp dword ptr [{b} + 0x22400C]";
        b = base(), x = x, z = z
    );
    unsafe {
        let d: u32 = game_read(SCRATCH_RVA);
        if (d.wrapping_sub(game_read::<u32>(RADIUS_RVA)) as i32) >= 0 {
            game_write(RADIUS_RVA, d);
        }
        d
    }
}

// ---- hook ----

fn regions() -> [Region; 2] {
    [at(SCRATCH_RVA), at(RADIUS_RVA)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, widen_object_radius(r.eax, r.ecx)))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = regions();
    let saved = capture(&all);
    for (x, z) in [
        (0u32, 0u32),
        (3, 4),
        (0xFFFF_FFFD, 4),
        (1000, 1),
        (0x7FFF, 0x7FFF),
        (5, 5),
    ] {
        for radius in [0u32, 5, 7, 0x8000_0000] {
            check_state(
                &mut t,
                &format!("point {x:#x} {z:#x}, radius {radius:#x}"),
                call_original,
                Regs {
                    eax: x,
                    ecx: z,
                    ..Regs::SENTINEL
                },
                &|| unsafe { game_write(RADIUS_RVA, radius) },
                &all,
                &rust,
            );
        }
    }
    restore(&all, &saved);
    t.finish()
}
