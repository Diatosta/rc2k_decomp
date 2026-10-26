//! `physics::ground_search` (0x441E2F): the course face under the point (`dword_639824`,
//! `dword_63982C`): `physics::try_last_face`, else `dword_5CCC74` = -1 and
//! `physics::search_cell` on the point's cell and its eight neighbours (`ground::CELLS`,
//! in the original's order) until one has it. `g_groundFound` (0x639830) 1 when found,
//! else 0. All inside `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; CF the found (no caller reads it: they read
//!      `g_groundFound`, `dword_5CCC74`, `dword_639828`)
//!
//! See `physics/ground.rs`. Caller: `physics::find_ground` (0x441E29), its only one.
//! Refuses (before anything) what `find_course_record` or `search_record_faces` would
//! refuse on any of the nine cells (also on cells past the one found).
//! Self-test: a copy of the original (its calls to copies) on the test's ground, records
//! for six cells, the test's points with last faces -1 and two real ones; found counted.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::ground::*;
use crate::physics::search_record_faces::{TEST_RECORDS, set_records};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::ground_search", rva(0x441E2F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the carry (the registers are kept).
pub unsafe fn ground_search() -> Result<bool, Refused> {
    let mut r = Regs::default();
    unsafe { ground_search_at(&Ground::game(), &mut r) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { ground_search() }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &Ground::game().regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// Record n's cell: (n % 3, n / 3), as `search_cell`'s test.
pub fn cell_key(n: u32) -> u32 {
    ((n / 3) << 11) + n % 3
}

/// The test's ground with `TEST_RECORDS` in cells (n % 3, n / 3) and the kind tables.
pub fn test_ground(seed: u32) -> TestGround {
    let mut test = TestGround::new(20, 25, TEST_RECORDS.len() as u32);
    test.lay_out(seed);
    set_records(&mut test, &cell_key);
    let g = test.ground();
    for n in 0..TEST_RECORDS.len() as u32 {
        let key = cell_key(n);
        let kind = (key & 0xF) + (((key >> 11) & 0xF) << 4);
        unsafe {
            g.put(0x5CC3E4 + 4 * kind, n);
            g.put(0x5CC7E4 + 4 * kind, n);
        }
    }
    test
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let test = test_ground(2);
    let copies = match test.copies() {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let g = test.ground();
    let mut found = 0;
    for (n, &(x, z)) in test_points().iter().enumerate() {
        for last in [u32::MAX, 5, 9] {
            let prepare = || unsafe {
                test.clear_work();
                test.set_point(x, z, 0x80, n % 2 == 0);
                g.put(0x5CCC78, last);
            };
            if check_case(
                &mut t,
                &format!("last face {last:#x}, point ({x:#x}, {z:#x})"),
                copies.ground_search.addr(),
                &test,
                Regs::SENTINEL,
                &prepare,
                &|g, r| unsafe { ground_search_at(g, r) },
                true,
            ) {
                found += 1;
            }
        }
    }
    t.check(found > 0, || "no case found the point".into());
    drop(copies);
    t.finish()
}
