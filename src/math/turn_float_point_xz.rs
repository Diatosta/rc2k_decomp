//! `math::turn_float_point_xz` (0x41FE74): a point of three floats in eax, ebx, ecx (as
//! dwords) turned in x and z: stored in the scratch point (`g_scratchX`, `Y`, `Z`),
//! `math::turn_scratch_xz` by edx, and loaded back. A sibling of `math::turn_point_xz`
//! without the whole numbers.
//!
//! In:  eax x, ebx y, ecx z (floats), edx the angle
//! Out: eax x', ebx y, ecx z' (floats), edx as `turn_scratch_xz` leaves it; the rest kept
//!      (esi, ebp pushed around); no flags read
//!
//! Callers: `sub_43F136`, `sub_45DA14`, `sub_45DC3E`, `sub_4B7058`.
//! Self-test: the original and the Rust version on the scratch point (put back after),
//! points and angles by lot, under three precisions.

use crate::errors::Refused;
use crate::framework::{
    EAX, EBX, ECX, EDX, GameHook, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva,
};
use crate::math::turn_scratch_xz::{
    X_RVA, Y_RVA, Z_RVA, point_floats, scratch, turn_scratch_xz, turn_scratch_xz_plain,
};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::floats::{FLOAT, Floats, Tolerance, check_state_floats, dispatch_state_floats};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("math::turn_float_point_xz", rva(0x41FE74), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: x', y, z' and the entry left in edx.
pub unsafe fn turn_float_point_xz(x: u32, y: u32, z: u32, angle: u32) -> [u32; 4] {
    unsafe { turn_float_with(x, y, z, angle, turn_scratch_xz) }
}

/// The converted version (`turn_scratch_xz_plain`).
pub unsafe fn turn_float_point_xz_plain(x: u32, y: u32, z: u32, angle: u32) -> [u32; 4] {
    unsafe { turn_float_with(x, y, z, angle, turn_scratch_xz_plain) }
}

unsafe fn turn_float_with(
    x: u32,
    y: u32,
    z: u32,
    angle: u32,
    turn: unsafe fn(u32) -> u32,
) -> [u32; 4] {
    unsafe {
        game_write(X_RVA, x);
        game_write(Y_RVA, y);
        game_write(Z_RVA, z);
        let entry = turn(angle);
        [game_read(X_RVA), game_read(Y_RVA), game_read(Z_RVA), entry]
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let [x, y, z, entry] = unsafe { turn_float_point_xz_plain(r.eax, r.ebx, r.ecx, r.edx) };
    Ok(Left::nothing()
        .with(EAX, x)
        .with(EBX, y)
        .with(ECX, z)
        .with(EDX, entry))
}

/// The results' places that come from floats.
const REGS: [(u8, Tolerance); 2] = [(EAX, FLOAT), (ECX, FLOAT)];

extern "C" fn dispatch(r: &mut Regs) {
    let floats = Floats {
        memory: &[(scratch(), FLOAT)],
        regs: &REGS,
    };
    dispatch_state_floats(&HOOK, r, call_original, &[scratch()], &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [scratch()];
    let saved = capture(&regions);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 1..=8u32 {
                let mut f = [0u32; 3];
                test_floats(seed * 29 + 3, &mut f);
                check_state_floats(
                    &mut t,
                    HOOK.name(),
                    &format!("control word {word:#x}, seed {seed}"),
                    call_original,
                    Regs {
                        eax: f[0],
                        ebx: f[1],
                        ecx: f[2],
                        edx: seed.wrapping_mul(0x2468_ACE1),
                        ..Regs::SENTINEL
                    },
                    &|| {},
                    &regions,
                    &point_floats(word, &[(scratch(), FLOAT)], &REGS),
                    &rust,
                );
            }
        });
    }
    restore(&regions, &saved);
    t.finish()
}
