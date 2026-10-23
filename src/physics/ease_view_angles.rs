//! `physics::ease_view_angles` (0x43846D): with the mode's record's change time (+0x3C)
//! not 0: the share done (`physics/camera_view.rs`) times the record's three angle
//! changes (+0x40 .. +0x48) x `flt_5C5D98` (`fistp`) << 16 added to `unk_71A694`,
//! `unk_71A698`, `unk_71A69C`.
//!
//! In:  nothing
//! Out: esi the mode's record; eax the last change added (or the time held, kept when
//!      the record has no time); the rest kept; no flags read
//!
//! Caller: `sub_437A53`.
//! Self-test: `physics/camera_view.rs` (`view_test_floats`), modes and times by lot,
//! exact at double precision; at extended a step may land one off (<< 16 in the angle).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, ESI, Regs};
use crate::physics::camera_view::{rd, record, record_plain, share_done, share_done_plain};
use crate::physics::flow::{base, g, p, x87_run};
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;
use rc2k_core::physics::camera_view::ease_angles;

/// The Rust version (the exact one, for `physics::update_camera`).
pub unsafe fn ease_view_angles(entry: &Regs) -> Left {
    let esi = record();
    let left = Left::nothing().with(ESI, esi);
    if rd(esi + 0x3C) == 0 {
        return left;
    }
    share_done(esi);
    let mut eax = entry.eax;
    for (k, to) in [0x71A694usize, 0x71A698, 0x71A69C].iter().enumerate() {
        x87_run!(
            "fld dword ptr [{s}]",
            "fmul dword ptr [{b} + 0x205260]",
            "fmul dword ptr [{b} + 0x1C5D98]",
            "fistp dword ptr [{b} + 0x22400C]";
            b = base(), s = esi + 0x40 + 4 * k as u32
        );
        eax = g(0x62400C) << 16;
        p(*to, g(*to).wrapping_add(eax));
    }
    left.with(EAX, eax)
}

/// The converted version (`rc2k_core::physics::camera_view`, plain floats): the hook's.
// Converted: `rc2k_core::physics::camera_view::{share_done, ease_angles}`.
pub unsafe fn ease_view_angles_plain(_: &Regs) -> Left {
    let esi = record_plain();
    let left = Left::nothing().with(ESI, esi);
    if rd(esi + 0x3C) == 0 {
        return left;
    }
    share_done_plain(esi);
    let changes = [0x40, 0x44, 0x48].map(|k| f32::from_bits(rd(esi + k)));
    let mut angles = ANGLES.map(g);
    let share = f32::from_bits(g(0x605260));
    let (n, added) = ease_angles(changes, share, f32::from_bits(g(0x5C5D98)), &mut angles);
    p(0x62400C, n as u32);
    for (x, v) in ANGLES.iter().zip(angles) {
        p(*x, v);
    }
    left.with(EAX, added)
}

/// The three angles (`unk_71A694` ..).
const ANGLES: [usize; 3] = [0x71A694, 0x71A698, 0x71A69C];

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { ease_view_angles_plain(r) })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
