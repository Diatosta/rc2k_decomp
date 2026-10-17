//! `marks::place_graded_points` (0x4BA16C): a spline record's first two grades of points
//! placed. For grade 0: the turns by the record's +0x20, +0x24, +0x28
//! (`splines::turn_record`), its +0xC points from its start (+0) turned into
//! `g_marksOrderedCoords`, bent (`marks::bend_points`), and each written plus +0xC54 ..
//! through the record's pointer for it (+0x454); for grade 1 the same with +0x2C for the y
//! turn, its start +4 and count +0x10, and no bend.
//!
//! In:  ebp a spline record (`g_marksSplinesC`)
//! Out: eax grade 1's count, the rest as its last point leaves them
//!
//! Callers: 0x4B99D4, 0x4B9A2C. Refuses (before anything) a negative count (the loop
//! runs on), points past the record's 0x80, a null pointer among them, an angle
//! `fsincos` cannot take, and `bend_points`' refusals. Self-test: a record of the test's
//! (`TestSpline`) whose pointers name a buffer of the test's; compared on all it may
//! write.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::bend_points::bend_points;
use crate::marks::splines::{
    CHOSEN_RVA, MADE_RVA, MOST_POINTS, SPLINE_LEN, TestSpline, transform_int_points_from,
    turn_record, turnable_record, work_regions, write_point,
};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "marks::place_graded_points",
    rva(0x4BA16C),
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

const SWITCH_RVA: usize = rva(0x7695CA); // dword_7695CA
const POINTERS: u32 = 0x454;
const COORDS: u32 = 0x654;
/// Per grade: its start's and count's offsets and its turn's.
const GRADES: [(u32, u32, [u32; 3]); 2] =
    [(0, 0xC, [0x20, 0x24, 0x28]), (4, 0x10, [0x20, 0x2C, 0x28])];

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// Why the Rust version refuses, if it does, worked out without writing.
unsafe fn refusal(s: u32) -> Result<(), Refused> {
    unsafe {
        for (start, count, turns) in GRADES {
            let (first, n) = (get(s + start), get(s + count) as i32);
            if n < 0 {
                return Err(BadInput("a negative grade count: the loop runs on").into());
            }
            if n > 0 && first.saturating_add(n as u32) > MOST_POINTS {
                return Err(BadInput("a grade's points past the record's 0x80").into());
            }
            if (0..n as u32).any(|k| get(s + POINTERS + 4 * (first + k)) == 0) {
                return Err(BadInput("a null point pointer").into());
            }
            turnable_record(s, turns)?;
        }
        if game_read::<u32>(SWITCH_RVA) != game_read::<u32>(CHOSEN_RVA)
            && get(s + 0x30) >= MOST_POINTS
        {
            return Err(BadInput("a spline point number of 0x80 or more").into());
        }
    }
    Ok(())
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn place_graded_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
        refusal(s)?;
        for (n, (start, count, turns)) in GRADES.into_iter().enumerate() {
            let (first, points) = (get(s + start), get(s + count));
            g = turn_record(&g, s, turns)?;
            g = transform_int_points_from(&g, s + COORDS + 12 * first, points)?;
            if n == 0 {
                g = bend_points(&g)?;
            }
            for k in 0..points {
                g = write_point(&g, s, k, k + first);
            }
            g.eax = points;
        }
    }
    Ok(g)
}

pub fn regions(spline: u32) -> Vec<Region> {
    let mut r = work_regions();
    r.push((spline, SPLINE_LEN as usize));
    r.extend(crate::marks::bend_points::regions(spline));
    for (start, count, _) in GRADES {
        let (first, n) = unsafe { (get(spline + start), get(spline + count) as i32) };
        for k in 0..n.clamp(0, MOST_POINTS as i32) as u32 {
            let i = first.wrapping_add(k);
            if i < MOST_POINTS {
                let p = unsafe { get(spline + POINTERS + 4 * i) };
                if p != 0 {
                    r.push((p, 12));
                }
            }
        }
    }
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { place_graded_points(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r.ebp);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let spline = TestSpline::new();
    let s = spline.addr();
    let targets = vec![0u32; 3 * MOST_POINTS as usize];
    let base = targets.as_ptr() as u32;
    let mut regions = work_regions();
    regions.push((s, SPLINE_LEN as usize));
    regions.extend(crate::marks::bend_points::regions(s));
    regions.push((base, 4 * targets.len()));
    let mut vars = regions.clone();
    vars.extend([SWITCH_RVA, CHOSEN_RVA, MADE_RVA].map(at));
    let saved = capture(&vars);
    // (grade 0's start and count, grade 1's, the switch, the chosen)
    for (g0, g1, switch, chosen) in [
        ((0u32, 5u32), (5u32, 7u32), 1u32, 0u32),
        ((0, 0), (0, 0), 0, 0),
        ((10, 0x40), (0x50, 0x30), 2, 1),
        ((3, 1), (0, 0x80), 1, 1),
    ] {
        check_state(
            &mut t,
            &format!("grade 0 {g0:x?}, grade 1 {g1:x?}, switch {switch}, chosen {chosen}"),
            call_original,
            Regs {
                ebp: s,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                spline.fill(g0.1 * 3 + g1.0 + switch, 0x80);
                for (o, v) in [
                    (0u32, g0.0),
                    (0xC, g0.1),
                    (4, g1.0),
                    (0x10, g1.1),
                    (0x30, 9),
                ] {
                    ((s + o) as *mut u32).write_unaligned(v);
                }
                for k in 0..MOST_POINTS {
                    ((s + POINTERS + 4 * k) as *mut u32)
                        .write_unaligned(base + 12 * ((k * 37 + 5) % MOST_POINTS));
                }
                std::ptr::write_bytes(base as *mut u8, 0xAA, 4 * targets.len());
                game_write(SWITCH_RVA, switch);
                game_write(CHOSEN_RVA, chosen);
                game_write(MADE_RVA, g0.0 * 0x11 + 3);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    drop(targets);
    t.finish()
}
