//! `race::place_view` (0x4389A8): the view's place and angles from the camera (see
//! `race/frame_draw.rs`): flat (`dword_60932C`) as it is, with `g_car` +0x6A8 moved by
//! `dword_5CCD80`; else moved back by `dword_5CCD80` turned by the camera's two angles
//! (`math::turn_point_xz_ecx`, `math::turn_point_xz`), and kept also as whole numbers and
//! floats (`dword_6062B0`, `dword_6062B4`, `flt_6062A4` ..).
//!
//! In:  nothing
//! Out: eax, ebx, ecx (and edx when turned) as it leaves them (its caller `sub_438325`
//!      jumps on to a `call`)
//!
//! Shadow mode: by state (the place, the angles' dwords, `g_car` +0x6A8, the scratch
//! point). Self-test: the camera's dwords set (places, distances, angles), flat and not;
//! all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { place_view(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
