//! `physics::crossed_edge_contact` (0x45DC3E): the contact of the probe point that
//! crossed a face's edge (`dword_6D3B84`, into `dword_6D3214`). Its figures from
//! `dword_6D2C14[n]`, `6D2E14[n]` (x, z before), `6D1E14[n]`, `6D2014[n]` (x, z now) and
//! the face `dword_6D2F14[n]` into `dword_6D31A0` .. `6D31B0`; the face's four corners'
//! x, z (scene points) into `dword_6D3180` .. `6D319C`. Which side of the probe's path
//! each corner lies (the sign of the 64-bit (now - before) cross product; the first
//! product kept in `qword_620460`) picks the edge crossed: p1 -> p2, p2 -> p4, p4 -> p3,
//! else p3 -> p1 (corner words +4 .. +0x10), into `dword_6141BC`, `6141C0`. The normal
//! across it (`physics::wall_edge_contact`'s `edge_normal`; the heading also into
//! `dword_6D31B4`), the speeds, the probe's point (y x `flt_5C5DD0`), no slide limit;
//! `physics::contact_force` into contact record 1 (`g_contactRecords` + 0x20),
//! `physics::place_sound_e1`.
//!
//! In:  nothing
//! Out: ebp the face, and the registers as the steps and `contact_force` leave them; no
//!      flags read (the caller goes on)
//!
//! Caller: `physics::crossed_edge`.
//! Refuses (before anything) a probe point past 0x3F (the tables' room), and what
//! `place_sound_e1` refuses (what was written put back).
//! Self-test: `physics/crossed_edge.rs` (`test_crossed`): the original and the Rust
//! version on the game's variables (saved and put back) with a test face and scene
//! points, paths across each edge, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::math::x87::mul;
use crate::physics::contact_force::{RECORD, contact_force};
use crate::physics::crossed_edge::regions;
use crate::physics::flow::{a, copy3, g, guard, p};
use crate::physics::scene_point_hits_car::SCENE_POINTS;
use crate::physics::touch_face::FACES;
use crate::physics::wall_edge_contact::edge_normal;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::walls;

/// `dword_6D3B84`: the probe point that crossed an edge (-1 for none).
pub const CROSSED: usize = 0x6D3B84;

fn rd(x: u32) -> u32 {
    unsafe { (x as *const u32).read_unaligned() }
}

/// Which side of the probe's path the point (x, z) lies: true for not left of it (the
/// 64-bit difference not below 0, `jns` / `stc`). `qword_620460` the first product, eax
/// the difference's low dword.
fn side(r: &mut Regs, x: u32, z: u32) -> bool {
    let (first, difference) = walls::path_side(
        (g(0x6D31A0), g(0x6D31A4)),
        (g(0x6D31A8), g(0x6D31AC)),
        (x, z),
    );
    p(0x620460, first as u32);
    p(0x620464, (first >> 32) as u32);
    r.eax = difference as u32;
    difference >= 0
}

/// The steps, not guarded (the probe checked before).
// Converted: `rc2k_core::physics::walls::{path_side, crossed_edge}`.
pub fn steps(r: &mut Regs) -> Result<(), Refused> {
    let n = g(CROSSED);
    r.ebx = n;
    p(0x6D3214, n);
    let k = n as usize;
    for (to, from) in [
        (0x6D31A0, 0x6D2C14),
        (0x6D31A4, 0x6D2E14),
        (0x6D31A8, 0x6D1E14),
        (0x6D31AC, 0x6D2014),
        (0x6D31B0, 0x6D2F14),
    ] {
        r.eax = g(from + 4 * k);
        p(to, r.eax);
    }
    r.ebx = g(0x6D31B0);
    let face = (r.ebx << 6).wrapping_add(g(FACES));
    r.ebp = face;
    for j in 0..4u32 {
        r.ebx = rd(face + 4 + 4 * j);
        r.esi = r.ebx.wrapping_mul(12).wrapping_add(g(SCENE_POINTS));
        r.eax = rd(r.esi);
        p(0x6D3180 + 8 * j as usize, r.eax);
        r.eax = rd(r.esi + 8);
        p(0x6D3184 + 8 * j as usize, r.eax);
    }
    r.esi = face;
    let corner = |j: usize| (g(0x6D3180 + 8 * j), g(0x6D3184 + 8 * j));
    // The original's order of tests (each computed, `qword_620460` the last one's).
    let (first, second) = walls::crossed_edge(|j| {
        let (x, z) = corner(j);
        side(r, x, z)
    });
    r.eax = rd(face + first);
    r.ebx = rd(face + second);
    p(0x6141BC, r.eax);
    p(0x6141C0, r.ebx);
    let [x, y, z, edx] = edge_normal(r.eax, r.ebx);
    p(0x6D31B4, g(0x6141C8));
    r.eax = x;
    r.ebx = y;
    r.ecx = z;
    r.edx = edx;
    copy3(0x6D30A8, 0x71A170);
    copy3(0x6D30B4, 0x71A15C);
    r.ebx = n;
    p(0x6D3090, g(0x6D2214 + 4 * k));
    p(0x6D3094, g(0x6D2314 + 4 * k));
    p(0x6D3098, g(0x6D2414 + 4 * k));
    p(0x6D3094, mul(g(0x6D3094), g(0x5C5DD0)));
    p(0x6D3100, 0);
    p(RECORD, a(0x614A70));
    unsafe { contact_force() }.apply(r);
    unsafe { crate::physics::place_sound_e1::place_sound_e1() }?;
    Ok(())
}

/// The probe checked: refused past 0x3F.
pub fn check_probe() -> Result<(), Refused> {
    Ok(walls::check_probe(g(CROSSED))?)
}

/// The Rust version.
pub unsafe fn crossed_edge_contact(entry: &Regs) -> Result<Left, Refused> {
    check_probe()?;
    guard(&regions(), || {
        let mut r = *entry;
        steps(&mut r)?;
        Ok(left_of(&r))
    })
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { crossed_edge_contact(r) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
