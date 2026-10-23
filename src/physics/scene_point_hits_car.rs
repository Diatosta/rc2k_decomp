//! `physics::scene_point_hits_car` (0x45E7BF): `physics::point_hits_car` for the scene
//! point whose number is the dword at esi+4 (`g_scenePoints` + 12 n), with no offset:
//! `dword_614108` + 1, `dword_6D3B40` .. = 0.
//!
//! In:  esi the record (+4 the point's number), ebp the object
//! Out: CF as `point_hits_car` leaves it; ebx the number, esi the point; the rest kept
//!
//! Caller: `sub_45E3BF` (0x45E48C, `jb`).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) with test points (`g_scenePoints` pointed at them for the test), the car's
//! figures by lot, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EBX, ESI, Regs};
use crate::physics::flow::{a, fill3, g, p};
use crate::physics::point_hits_car::{self, point_hits_car};
use crate::screens::harness::Left;
use crate::service::harness::{Region, state_entry};

/// `g_scenePoints`: the pointer to the scene's points (three whole numbers each).
pub const SCENE_POINTS: usize = 0x5CCCC0;

/// The Rust version: (inside, ebx, esi).
pub unsafe fn scene_point_hits_car(record: u32, object: u32) -> (bool, u32, u32) {
    let number = unsafe { ((record + 4) as *const u32).read_unaligned() };
    let point = number.wrapping_mul(12).wrapping_add(g(SCENE_POINTS));
    p(0x614108, g(0x614108).wrapping_add(1));
    fill3(0x6D3B40, 0);
    (unsafe { point_hits_car(point, object) }, number, point)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (inside, ebx, esi) = unsafe { scene_point_hits_car(r.esi, r.ebp) };
    Ok(Left::carry(inside).with(EBX, ebx).with(ESI, esi))
}

pub fn regions() -> Vec<Region> {
    let mut all = point_hits_car::regions();
    all.extend([(a(0x614108), 4), (a(0x6D3B40), 12)]);
    all
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
