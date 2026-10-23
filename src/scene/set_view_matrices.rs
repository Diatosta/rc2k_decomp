//! `scene::set_view_matrices` (0x438FBF): the view's matrices from its angles and place.
//!
//! In:  nothing
//! Out: eax, ebx = the place's x and y negated, ecx = edx = 0, esi = the view's
//!      turning, edi = that undone, ebp = the view's matrix
//!
//! `g_viewTurnMatrix` = the identity `unk_5E9F4C`; the fine turnings about y, x and z
//! for `dword_6062B8`, `dword_6062BC`, `dword_6062C0`, each negated; y times x to
//! `g_litMatrix` (used as work), that times z to `g_viewTurnMatrix`; `scene::set_place`
//! with `dword_606298` .. negated, and `g_placedMatrix` copied to `unk_5EA1CC`; the
//! placed matrix times the turning to `g_viewMatrix`; `scene::invert_turn` of the
//! turning to `flt_5EA20C`.
//! Two callers: `scene::prepare_menu_scene` and the front end's 3D set-up.
//! Self-test: 40 settings by lot on the game's matrices, put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `set_view_matrices_plain`, through
//! `rc2k_core::scene::matrices::set_view_matrices` (plain floats; the x87 version stays for
//! the Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::scene::view_matrix::{self};
use crate::screens::harness::Left;
use crate::service::floats::*;

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { view_matrix::set_view_matrices_plain() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
