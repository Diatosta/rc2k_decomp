//! `physics::count_ground_probe` (0x438E22): one ground probe, counted: the face mask
//! `dword_620470` 0x40, `dword_608C3C` up by one, `physics::find_ground`; when found
//! `dword_608C40` up by one, and when the face is the last one again (`dword_5CCC78` =
//! `dword_5CCC74`) `dword_608C44` too. A reading: the camera's (or a test drive's) ground
//! statistics.
//!
//! In:  the point (`dword_639824`, `dword_63982C`; the callers set it)
//! Out: as `find_ground` leaves them, eax `dword_5CCC78` when found; no flags read (the
//!      callers compare `g_groundFound`)
//!
//! Callers: `sub_438BF4`, `sub_438CEC` (three).
//! Refuses what `find_ground` refuses. Shadow mode runs the original alone
//! (`physics/reset_parts.rs`).
//! Self-test: a copy whose call goes to a stand-in (place 18), against the Rust version
//! with the same: found or not, the same face or not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::physics::reset_parts::{FIND_GROUND, both_with, copy_of, dispatch_reset, regions};
use crate::race::car_starts::{Case, Part, prepare};
use crate::service::harness::{Region, capture, restore};
use rc2k_core::physics::walls::ProbeCounts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::count_ground_probe",
    rva(0x438E22),
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

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// The Rust version, `find_ground` through `part` (place 18).
// Converted: `rc2k_core::physics::walls::ProbeCounts`.
pub unsafe fn count_ground_probe(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let mut counts = ProbeCounts {
        tries: g(0x608C3C),
        found: g(0x608C40),
        same: g(0x608C44),
    };
    p(0x620470, 0x40);
    counts.tried();
    p(0x608C3C, counts.tries);
    part(FIND_GROUND, &mut r)?;
    let found = g(0x639830) != 0;
    if found {
        // the original's `mov eax, dword_5CCC78` for its compare
        r.eax = g(0x5CCC78);
        counts.searched(true, r.eax, g(0x5CCC74));
        p(0x608C40, counts.found);
        p(0x608C44, counts.same);
    }
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_reset(&HOOK, r, call_original, count_ground_probe);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x438E22,
        0x38,
        &[0xC7, 0x05, 0x70, 0x04, 0x62, 0x00, 0x40, 0x00, 0x00, 0x00],
        &[0x10],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let a = |x: usize| game_addr(rva(x)) as u32;
    let extra: [Region; 4] = [
        (a(0x620470), 4),
        (a(0x608C3C), 12),
        (a(0x639830), 4),
        (a(0x5CCC74), 8),
    ];
    let mut all = regions();
    all.extend_from_slice(&extra);
    let saved = capture(&all);
    for (found, same) in [(0u32, true), (1, false), (1, true), (2, true)] {
        let fill = || {
            prepare(
                &Case {
                    mode: 2,
                    no_place: 0,
                    side: 0,
                    cars_2: 1,
                    current: 2,
                    cars: 4,
                },
                found * 3 + same as u32,
            );
            p(0x639830, found);
            p(0x5CCC74, 0x55);
            p(0x5CCC78, if same { 0x55 } else { 0x56 });
            p(0x608C3C, 10);
            p(0x608C40, 20);
            p(0x608C44, 30);
        };
        both_with(
            &mut t,
            &format!("found {found}, same face {same}"),
            &copy,
            0,
            Regs::SENTINEL,
            &fill,
            count_ground_probe,
            &extra,
        );
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
