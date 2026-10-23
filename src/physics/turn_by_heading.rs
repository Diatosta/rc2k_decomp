//! `physics::turn_by_heading` (0x43F136): the floats at esi (x) and esi+8 (z) turned by
//! minus the work car's heading (`g_car` +0x9C0, `unk_71A998`) with
//! `math::turn_float_point_xz` (ebx passed through as y) and stored back.
//!
//! In:  esi the point, ebx passed on
//! Out: eax x', ecx z', edx as `turn_scratch_xz` leaves it; the rest kept; no flags read
//!
//! Caller: `sub_45E800` (0x45E8A9).
//! Self-test: the original and the Rust version on the game's variables (the heading and
//! the scratch point saved and put back) and a test point, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, EDX, Regs, game_read, rva};
use crate::math::turn_float_point_xz::turn_float_point_xz;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const HEADING_RVA: usize = rva(0x71A998); // g_car + 0x9C0

/// The Rust version.
pub unsafe fn turn_by_heading(point: u32, ebx: u32) -> Left {
    unsafe {
        let p = point as *mut u32;
        let angle = game_read::<u32>(HEADING_RVA).wrapping_neg();
        let [x, _, z, entry] =
            turn_float_point_xz(p.read_unaligned(), ebx, p.add(2).read_unaligned(), angle);
        p.write_unaligned(x);
        p.add(2).write_unaligned(z);
        Left::nothing().with(EAX, x).with(ECX, z).with(EDX, entry)
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { turn_by_heading(r.esi, r.ebx) })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
