//! `marks::span_ring` (0x4BE469): a shape's ring of quads spanned into the mask's runs, its
//! angles kept at esi.
//!
//! In:  ebp the shape (+4 the quads), esi the angles' table
//! Out: eax the spans so far, esi 8 a quad on
//!
//! See `marks/rings.rs`; `sub_4BE592` (`span_ring_quad`) is ported inside it (not hooked:
//! it reads the quad's number from its caller's stack), and the quad it works out is thrown
//! away for the corner table's (section 16). Callers: `sub_4BDE9A` (`marks::make_masks`,
//! the two shapes). Refuses a count of 0 or less or past 0x80, and runs or spans past their
//! room, before writing. Self-test: the original and the Rust version on shapes and corner
//! tables by lot in a work block of the test's, the refusals; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::marks::rings::*;
use crate::marks::spans::{TestWork, test_corners};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::span_ring", rva(0x4BE469), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { span_ring(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = ring_regions();
    let read = |rv: usize| unsafe { crate::framework::game_read::<u32>(rv) };
    let n = unsafe { (r.ebp.wrapping_add(4) as *const u32).read_unaligned() }.min(MOST_QUADS);
    regions.extend([
        (r.ebp, 0x24),
        (r.esi, 8 * n as usize + 8),
        {
            // The rest of the work block from the cursor.
            let cursor = read(crate::marks::masks::CURSOR_RVA);
            let end = read(crate::marks::masks::WORK_RVA).wrapping_add(0x80000);
            (cursor, end.saturating_sub(cursor).min(0x80000) as usize)
        },
        (read(crate::marks::masks::RUNS_RVA), 0x20 * n as usize),
    ]);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let work = TestWork::new();
    let extra: Vec<Region> = vec![
        at(CORNERS_RVA),
        at(crate::marks::masks::RUN_COUNT_RVA),
        (game_addr(QUAD_RVA) as u32, 0x54),
    ];
    let saved = capture(&extra);
    let mut shape = [0u32; 9];
    let sp = shape.as_mut_ptr() as u32;
    let mut angles = [0u32; 2 * 9];
    let ap = angles.as_mut_ptr() as u32;
    let mut regions = work.regions();
    regions.extend(extra.iter().copied());
    regions.extend([(sp, 0x24), (ap, angles.len() * 4)]);
    for (n, seed) in [(1u32, 1u32), (2, 2), (4, 3), (8, 4), (8, 5)] {
        let corners = test_corners(seed, 2 * (n as usize + 1));
        // Entries of 0x18: two corners at +0 and +0xC.
        let mut table = vec![0u32; 6 * (n as usize + 1)];
        for e in 0..=n as usize {
            table[6 * e] = corners[4 * e];
            table[6 * e + 1] = corners[4 * e + 1];
            table[6 * e + 3] = corners[4 * e + 2];
            table[6 * e + 4] = corners[4 * e + 3];
        }
        check_state(
            &mut t,
            &format!("{n} quads, seed {seed}"),
            call_original,
            Regs {
                ebp: sp,
                esi: ap,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                work.reset();
                let mut v = [0u32; 9];
                crate::scene::matrix3::test_floats(seed + 20, &mut v);
                v[1] = n;
                v[2] = 0xAAAA_AAAA;
                (sp as *mut [u32; 9]).write(v);
                std::ptr::write_bytes(ap as *mut u8, 0xAA, 4 * 18);
                game_write(CORNERS_RVA, table.as_ptr() as u32);
            },
            &regions,
            &rust,
        );
    }
    for n in [0u32, MOST_QUADS + 1] {
        work.reset();
        unsafe { (sp as *mut u32).add(1).write(n) };
        t.check(
            unsafe {
                span_ring(&Regs {
                    ebp: sp,
                    esi: ap,
                    ..Regs::SENTINEL
                })
            }
            .is_err(),
            || format!("{n:#x} quads: the Rust version went on"),
        );
    }
    restore(&extra, &saved);
    drop(work);
    t.finish()
}
