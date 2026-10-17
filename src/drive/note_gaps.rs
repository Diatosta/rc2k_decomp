//! `drive::note_gap_behind` and `drive::note_gap_ahead`' shared core and test: a car's
//! (esi) checkpoint (+0x98) against the lead car's (car block 0's +0x98, `dword_70F484`),
//! the difference plus one kept when below the smallest so far (unsigned).

use crate::framework::{EDX, RawHook, Regs, SelfTest, call_game};
use crate::screens::harness::Left;

/// edx = (behind: lead - car, else car - lead) + 1; into `best` when below it.
pub unsafe fn note_gap_at(car: u32, lead: u32, best: u32, behind: bool) -> Left {
    unsafe {
        let mine = ((car + 0x98) as *const u32).read_unaligned();
        let theirs = (lead as *const u32).read_unaligned();
        let d = if behind {
            theirs.wrapping_sub(mine)
        } else {
            mine.wrapping_sub(theirs)
        };
        let edx = d.wrapping_add(1);
        let b = best as *mut u32;
        if edx < b.read_unaligned() {
            b.write_unaligned(edx);
        }
        Left::nothing().with(EDX, edx)
    }
}

/// The test of either: a copy whose lead and best are the test's, on a test car.
pub fn test_gap(
    hook: &RawHook,
    address: usize,
    entry: &[u8],
    best_var: usize,
    behind: bool,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 0x100 / 4 + 2];
    let car = cells.as_mut_ptr() as u32;
    let (lead, best) = (car + 0x100, car + 0x104);
    let copy = match crate::audio::test_copy::make(
        hook,
        &crate::audio::test_copy::Copy {
            address,
            len: 0x1C,
            entry,
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x70F484, lead), (best_var, best)],
        3,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for (mine, theirs, b) in [
        (10u32, 5u32, 100u32),
        (5, 10, 100),
        (5, 10, 3),
        (7, 7, 1),
        (7, 7, 2),
        (0, 0xFFFF_FFFF, u32::MAX),
        (0x8000_0000, 1, 0x7FFF_FFFF),
    ] {
        let fill = || unsafe {
            ((car + 0x98) as *mut u32).write(mine);
            (lead as *mut u32).write(theirs);
            (best as *mut u32).write(b);
        };
        fill();
        let before = Regs {
            esi: car,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs_best = unsafe { (best as *const u32).read() };
        fill();
        let left = unsafe { note_gap_at(car, lead, best, behind) };
        let mine_best = unsafe { (best as *const u32).read() };
        let ok = left.matches(&before, &ro) && theirs_best == mine_best;
        t.check(ok, || {
            format!("car {mine:#x}, lead {theirs:#x}, best {b:#x}: original {theirs_best:#x} edx {:#x}; rust {mine_best:#x} {left:?}", ro.edx)
        });
    }
    drop((copy, cells));
    t.finish()
}
