//! `marks::collect_points_b` (0x4BD59F): face edi's points added to points list B
//! (`g_marksPointsB`), each once with the face, up to 0x40.
//!
//! In:  edi the face
//! Out: eax the count, ebx the last point read, ecx where the last search stopped, ebp past
//!      the last point taken in
//!
//! See `marks/masks.rs`. One caller, `sub_4BD1A1` (0x4BD2A5). Refuses a list of more than
//! 0x40. Self-test: the original and the Rust version on a face of the test's (points
//! repeated, the list filling up), the refusal; the list put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::masks::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::collect_points_b", rva(0x4BD59F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { collect_points(r, &POINTS_B) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &POINTS_B.regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

const FACE_POINTS: u32 = 12;

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = POINTS_B.regions();
    let saved = capture(&regions);
    let face = vec![0u32; (4 + 4 * FACE_POINTS) as usize];
    let f = face.as_ptr() as u32;
    for had in [0u32, 1, 0x20, 0x3A, MOST_POINTS] {
        for count in [0u32, 5, FACE_POINTS] {
            let prepare = || unsafe {
                let mut s = had * 7 + count;
                for n in 0..face.len() as u32 {
                    s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
                    ((f + 4 * n) as *mut u32).write_unaligned(s >> 4);
                }
                (f as *mut u32).write_unaligned(count);
                for k in 0..FACE_POINTS {
                    ((f + 0x10 + 0x10 * k) as *mut u32).write_unaligned(0x30 + k % 9);
                }
                let mut ids = [0u32; MOST_POINTS as usize];
                for (k, v) in ids.iter_mut().enumerate() {
                    *v = k as u32;
                }
                game_write(POINTS_B.ids, ids);
                game_write(POINTS_B.faces, [0xAAAA_AAAAu32; MOST_POINTS as usize]);
                game_write(POINTS_B.count, had);
            };
            check_state(
                &mut t,
                &format!("{had:#x} in the list, a face of {count}"),
                call_original,
                Regs {
                    edi: f,
                    ..Regs::SENTINEL
                },
                &prepare,
                &regions,
                &rust,
            );
        }
    }
    unsafe { game_write(POINTS_B.count, MOST_POINTS + 1) };
    t.check(
        unsafe {
            collect_points(
                &Regs {
                    edi: f,
                    ..Regs::SENTINEL
                },
                &POINTS_B,
            )
        }
        .is_err(),
        || "0x41 in the list: the Rust version went on".into(),
    );
    restore(&regions, &saved);
    t.finish()
}
