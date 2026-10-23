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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::marks::blend_points::blend_points;
use crate::marks::splines::{CHOSEN_RVA, MADE_RVA, MOST_POINTS, ORDERED_COORDS_RVA};
use crate::math::x87_run::ext_to_int;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, state_entry};

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
