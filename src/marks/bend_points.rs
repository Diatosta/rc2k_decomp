//! `marks::bend_points` (0x4BA646): unless `dword_7695CA` is `g_marksSplineChosen`, the
//! first record ebp +0xC ordered points (`g_marksOrderedCoords`) bent: A the record's point
//! +0x30 (into `dword_76A1D2` ..), C (+0x44 made 0x2EE0, or -16000 unless `dword_7695CA` is
//! 1; +0x48, +0x4C) into `dword_76A1EA` .., B (C's x, A's y, C's z) into `dword_76A1DE` ..;
//! D their blend by `g_marksSplinesMade`'s weights (`marks::blend_points`), rounded
//! (`fistp`, z first) into `dword_76A1F6` ..; each point less A plus D.
//!
//! In:  ebp a spline record
//! Out: eax `dword_7695CA` when nothing is done; else esi the point times 12, edx the
//!      weights' entry times 12, edi past the points, eax .. ecx the last point (or the
//!      addresses of A, B and C with none); the rest kept
//!
//! Caller: `sub_4BA16C`. Refuses (before anything) a point number of 0x80 or more and a
//! count above 0x80. Self-test: a record of the test's (`TestSpline`), several points,
//! counts and switches; compared on all it writes and the registers.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::marks::blend_points::blend_points;
use crate::marks::splines::{
    CHOSEN_RVA, MADE_RVA, MOST_POINTS, ORDERED_COORDS_RVA, SPLINE_LEN, TestSpline,
};
use crate::math::x87_run::ext_to_int;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::bend_points", rva(0x4BA646), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SWITCH_RVA: usize = rva(0x7695CA); // dword_7695CA
const A_RVA: usize = rva(0x76A1D2);
const B_RVA: usize = rva(0x76A1DE);
const C_RVA: usize = rva(0x76A1EA);
const D_RVA: usize = rva(0x76A1F6);
const WORK_LEN: usize = 0x30;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn bend_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
        g.eax = game_read(SWITCH_RVA);
        if g.eax == game_read::<u32>(CHOSEN_RVA) {
            return Ok(g);
        }
        let point = get(s + 0x30);
        let count = get(s + 0xC) as i32;
        if point >= MOST_POINTS {
            return Err(BadInput("a spline point number of 0x80 or more").into());
        }
        if count > MOST_POINTS as i32 {
            return Err(BadInput("more than 0x80 points to bend").into());
        }
        g.esi = point * 12;
        let p = s + 0x654 + g.esi;
        let a = [get(p), get(p + 4), get(p + 8)];
        game_write(A_RVA, a);
        let height: u32 = if g.eax == 1 { 0x2EE0 } else { 0xFFFF_C180 };
        ((s + 0x44) as *mut u32).write_unaligned(height);
        let c = [height, get(s + 0x48), get(s + 0x4C)];
        game_write(C_RVA, c);
        let b = [c[0], a[1], c[2]];
        game_write(B_RVA, b);
        let at = |r: usize| game_addr(r) as u32;
        let (v, left) = blend_points(&Regs {
            eax: at(A_RVA),
            ebx: at(B_RVA),
            ecx: at(C_RVA),
            edx: game_read(MADE_RVA),
            ..g
        });
        g = left;
        let d = [ext_to_int(v[0]), ext_to_int(v[1]), ext_to_int(v[2])];
        game_write(D_RVA, d);
        g.edi = at(ORDERED_COORDS_RVA);
        for _ in 0..count.max(0) {
            let q = [get(g.edi), get(g.edi + 4), get(g.edi + 8)];
            let r: [u32; 3] =
                std::array::from_fn(|k| q[k].wrapping_sub(a[k]).wrapping_add(d[k] as u32));
            for (k, x) in r.iter().enumerate() {
                ((g.edi + 4 * k as u32) as *mut u32).write_unaligned(*x);
            }
            [g.eax, g.ebx, g.ecx] = r;
            g.edi += 12;
        }
    }
    Ok(g)
}

pub fn regions(spline: u32) -> Vec<Region> {
    vec![
        (game_addr(A_RVA) as u32, WORK_LEN),
        (spline + 0x44, 4),
        (
            game_addr(ORDERED_COORDS_RVA) as u32,
            12 * MOST_POINTS as usize,
        ),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { bend_points(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(r.ebp), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let spline = TestSpline::new();
    let s = spline.addr();
    let regions = regions(s);
    let mut vars = regions.clone();
    vars.extend([SWITCH_RVA, CHOSEN_RVA, MADE_RVA].map(at));
    vars.push((s, SPLINE_LEN as usize));
    let saved = capture(&vars);
    for (switch, chosen, point, count) in [
        (1u32, 0u32, 0u32, 5i32),
        (0, 1, 7, 0x80),
        (2, 2, 3, 4),
        (1, 3, 0x7F, 0),
        (5, 0, 12, -3),
        (1, 0, 40, 1),
    ] {
        check_state(
            &mut t,
            &format!("switch {switch}, chosen {chosen}, point {point}, count {count}"),
            call_original,
            Regs {
                ebp: s,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                spline.fill(point * 7 + switch, 0x80);
                ((s + 0x30) as *mut u32).write_unaligned(point);
                ((s + 0xC) as *mut i32).write_unaligned(count);
                game_write(SWITCH_RVA, switch);
                game_write(CHOSEN_RVA, chosen);
                game_write(MADE_RVA, point * 0x31 + switch);
                for k in 0..3 * MOST_POINTS as usize {
                    game_write(
                        ORDERED_COORDS_RVA + 4 * k,
                        (k as u32).wrapping_mul(0x3_1415) as i32 >> 8,
                    );
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
