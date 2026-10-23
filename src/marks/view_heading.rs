//! `marks::view_heading` (0x4C4CEC): the view's heading: the angle between its forward
//! direction's level part (`flt_5EA3CC` +8, 0, +0x28, into `flt_91FF7E`) and
//! `flt_91FF8A` (`scene::angle_between_vectors`, both normalised in place), into
//! `flt_8F3975`; 2 pi less it when the level part's x (normalised) is not negative.
//!
//! In:  nothing
//! Out: esi 0x91FF7E, edi 0x91FF8A; the rest kept
//!
//! One caller, `sub_4C3EEE`. Self-test: the original and the Rust version on several
//! views and second vectors; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `view_heading_plain`, through
//! `rc2k_core::marks::view::heading` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::rounded;
use crate::scene::vector3::{angle_between_vectors, angle_between_vectors_plain};
use crate::screens::harness::Left;
use crate::service::floats::*;

const VIEW_RVA: usize = rva(0x5EA3CC); // flt_5EA3CC: the view's matrix
const LEVEL_RVA: usize = rva(0x91FF7E); // flt_91FF7E: three floats
const OTHER_RVA: usize = rva(0x91FF8A); // flt_91FF8A: three floats
const HEADING_RVA: usize = rva(0x8F3975); // g_marksViewHeading

/// The Rust version (see the header).
pub unsafe fn view_heading(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let x = x87::loaded(game_read(VIEW_RVA + 8));
        let z = x87::loaded(game_read(VIEW_RVA + 0x28));
        game_write(LEVEL_RVA, [x, 0, z]);
        g.esi = game_addr(LEVEL_RVA) as u32;
        g.edi = game_addr(OTHER_RVA) as u32;
        let mut heading = rounded(angle_between_vectors(g.esi, g.edi));
        if game_read::<u32>(LEVEL_RVA) & 0x8000_0000 == 0 {
            heading = x87::two_pi_less(heading);
        }
        game_write(HEADING_RVA, heading);
    }
    g
}

/// The converted version (plain floats): the hook's. The angle through the converted
/// `scene::angle_between_vectors`.
// Converted: `rc2k_core::marks::view::heading`
pub unsafe fn view_heading_plain(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        use rc2k_core::scene::places::loaded;
        let x = loaded(game_read(VIEW_RVA + 8));
        let z = loaded(game_read(VIEW_RVA + 0x28));
        game_write(LEVEL_RVA, [x, 0, z]);
        g.esi = game_addr(LEVEL_RVA) as u32;
        g.edi = game_addr(OTHER_RVA) as u32;
        let angle = rounded(angle_between_vectors_plain(g.esi, g.edi));
        let level_x = f32::from_bits(game_read(LEVEL_RVA));
        let heading = rc2k_core::marks::view::heading(f32::from_bits(angle), level_x);
        game_write(HEADING_RVA, heading.to_bits());
    }
    g
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { view_heading_plain(r) }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
