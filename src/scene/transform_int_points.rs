//! `scene::transform_int_points` (0x465829): ecx points of three whole numbers from esi
//! turned by the 3 x 3 matrix at ebp into edi.
//!
//! In:  ecx the count, esi from, edi to, ebp the matrix
//! Out: ecx -1 (one less with a negative count), esi and edi past the points
//!
//! See `scene/matrix3.rs`. Callers: `sub_4BA0AE`, `sub_4BA16C`, `sub_4BD1A1`, `sub_4BD5ED`,
//! `sub_4C1CC8`, `sub_4C2AA5` and code outside a function. Refuses ecx 0x80000000. Manual
//! patch with a private copy (0x53 bytes): its first instruction jumps to the loop's test,
//! and the loop starts at the third byte. Self-test: the original and the Rust version on
//! points and matrices by lot (apart and in place), the refusal.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::*;
use crate::screens::harness::Left;
use crate::service::floats::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::transform_int_points",
    rva(0x465829),
    thunk,
    &ORIGINAL,
)
.private_copy(0x53)
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { transform_int_points_plain(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(r.edi, 12 * (r.ecx as i32).max(0) as usize)];
    let memory = [(regions[0], ROUNDED)];
    let floats = Floats {
        memory: &memory,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions, &floats, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

const POINTS: u32 = 8;

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let buffer = vec![0u32; (9 + 6 * POINTS) as usize];
    let b = buffer.as_ptr() as u32;
    let (from, apart) = (b + 0x24, b + 0x24 + 12 * POINTS);
    let tolerant = Floats {
        memory: &[((b, buffer.len() * 4), ROUNDED)],
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, floats| {
        for count in [0u32, 1, POINTS, 0xFFFF_FFFE] {
            for seed in 1..=3u32 {
                for (what, to) in [("apart", apart), ("in place", from)] {
                    check_state_floats(
                        &mut t,
                        "scene::transform_int_points",
                        &format!("control word {word:#x}, {count:#x} points, seed {seed}, {what}"),
                        call_original,
                        Regs {
                            ecx: count,
                            esi: from,
                            edi: to,
                            ebp: b,
                            ..Regs::SENTINEL
                        },
                        &|| unsafe {
                            let mut m = [0u32; 9];
                            test_floats(seed, &mut m);
                            std::ptr::copy(m.as_ptr(), b as *mut u32, 9);
                            let mut s = seed;
                            for n in 0..6 * POINTS {
                                s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
                                let v = match s % 7 {
                                    0 => (s as i32) >> 4,
                                    _ => ((s >> 8) % 0x4_0000) as i32 - 0x2_0000,
                                };
                                ((from + 4 * n) as *mut i32).write_unaligned(v);
                            }
                        },
                        &[(b, buffer.len() * 4)],
                        floats,
                        &rust,
                    );
                }
            }
        }
    });
    t.check(
        unsafe {
            transform_int_points_plain(&Regs {
                ecx: 0x8000_0000,
                ..Regs::SENTINEL
            })
        }
        .is_err(),
        || "ecx 0x80000000: the Rust version went on".into(),
    );
    t.finish()
}
