//! `scene::turn_matrix3_y` (0x4657A6): the 3 x 3 matrix at esi a turn about Y by the angle in
//! st(0) (popped).
//!
//! In:  esi the matrix, st(0) the angle
//! Out: every register kept, the angle popped
//!
//! See `scene/matrix3.rs`. Thunk by `angle_hook!`: the angle is kept for the Rust version
//! and loaded again for the original. Refuses an angle `fsincos` cannot take (finite, 2^63
//! and more). Self-test: the original and the Rust version on angles from
//! floats and on 80-bit ones, the refusal.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, rva};
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::*;
use crate::screens::harness::Left;
use crate::service::floats::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static ANGLE: Angle = Angle::new();

pub static HOOK: RawHook =
    RawHook::new("scene::turn_matrix3_y", rva(0x4657A6), thunk, &ORIGINAL).self_test(self_test);

angle_hook!(
    thunk,
    call_original,
    original_with_angle,
    dispatch = dispatch,
    original = ORIGINAL,
    angle = ANGLE
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { turn_matrix3_plain(Axis::Y, r.esi, ANGLE.get()) }?;
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(r.esi, 0x24)];
    let memory = [(regions[0], FLOAT)];
    let floats = Floats {
        memory: &memory,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, original_with_angle, &regions, &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut buffer = [0xAAu8; 0x28];
    let len = buffer.len();
    let b = buffer.as_mut_ptr() as u32;
    let floats: [f32; 10] = [
        0.0,
        -0.0,
        0.25,
        -1.0,
        std::f32::consts::PI,
        -4.5,
        100.0,
        1.0e5,
        f32::INFINITY,
        f32::NAN,
    ];
    let angles = floats
        .iter()
        .map(|f| (format!("{f}"), extended(f.to_bits())))
        .chain(
            (-9..=9)
                .step_by(3)
                .map(|n| (format!("pi {n} / 8"), extended_pi(n))),
        )
        .collect::<Vec<_>>();
    let tolerant = Floats {
        memory: &[((b, len), FLOAT)],
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, floats| {
        for (what, angle) in angles.iter().cloned() {
            check_state_floats(
                &mut t,
                "scene::turn_matrix3_y",
                &format!("control word {word:#x}, angle {what}"),
                original_with_angle,
                Regs {
                    esi: b + 2,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    std::ptr::write_bytes(b as *mut u8, 0xAA, len);
                    ANGLE.set(angle);
                },
                &[(b, len)],
                floats,
                &rust,
            );
        }
    });
    t.check(
        unsafe { turn_matrix3_plain(Axis::Y, b, extended(1.0e19f32.to_bits())) }.is_err(),
        || "angle 1e19: the Rust version went on".into(),
    );
    t.finish()
}
