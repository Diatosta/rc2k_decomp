//! `physics::search_cell` (0x441FBA): the grid cell of (eax, ecx) (each >> 21; key =
//! (z cell << 11) + x cell), its course record by `physics::find_course_record`, then its
//! faces by `physics::search_record_faces`. CF set when a face has the point; clear when
//! not, or when the cell has no record.
//!
//! In:  eax x, ecx z (whole numbers, the point's plus or less a cell)
//! Out: CF; ecx the z cell << 11; the rest as `find_course_record` and (when a record is
//!      found) `search_record_faces` leave them
//!
//! See `physics/ground.rs`. Caller: `sub_441E2F` (nine cells around the point; `jb`).
//! Refuses what `find_course_record` and `search_record_faces` refuse (both before
//! anything).
//! Self-test: a copy of the original (its calls to copies of the two) on the test's
//! ground with records for six cells of the grid, the test's points and their
//! neighbours.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::ground::*;
use crate::physics::search_record_faces::{TEST_RECORDS, set_records};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::search_cell", rva(0x441FBA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: `r` as the original leaves it, and the carry.
pub unsafe fn search_cell(r: &mut Regs) -> Result<bool, Refused> {
    unsafe { search_cell_at(&Ground::game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    let carry = unsafe { search_cell(&mut g) }?;
    Ok(left_of(&g, carry))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &Ground::game().regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// Record n's cell: (n % 3, n / 3).
fn cell_key(n: u32) -> u32 {
    ((n / 3) << 11) + n % 3
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut test = TestGround::new(20, 25, TEST_RECORDS.len() as u32);
    let copies = match test.copies() {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    test.lay_out(1);
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
    let mut found = 0;
    for (n, &(x, z)) in test_points().iter().enumerate() {
        for (dx, dz) in [
            (0u32, 0u32),
            (0x20_0000, 0),
            (0, 0x20_0000),
            (0xFFE0_0000, 0),
        ] {
            let (cx, cz) = (x.wrapping_add(dx), z.wrapping_add(dz));
            let prepare = || {
                test.clear_work();
                test.set_point(x, z, 0, n % 2 == 1);
            };
            if check_case(
                &mut t,
                &format!("point ({x:#x}, {z:#x}), cell of ({cx:#x}, {cz:#x})"),
                copies.search_cell.addr(),
                &test,
                Regs {
                    eax: cx,
                    ecx: cz,
                    ..Regs::SENTINEL
                },
                &prepare,
                &|g, r| unsafe { search_cell_at(g, r) },
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
