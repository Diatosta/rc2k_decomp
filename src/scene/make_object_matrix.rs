//! `scene::make_object_matrix` (0x427812): an object's matrix from its angles, place
//! and size.
//!
//! In:  nothing
//! Out: eax = the place's x, ebx = the size, ecx = 0, edx = 0, esi = `flt_5EA10C`,
//!      edi = `flt_5EA04C`, ebp = the matrix made, `flt_5EA30C`
//!
//! `scene::set_light_direction`; the turning about y for `dword_635040` negated and
//! about x for `dword_63503C` negated; the place `dword_63501C` .. as floats
//! (`scene::set_place`); the y turning times the x turning to `flt_5EA10C`; all
//! sixteen of its floats times the size `dword_635038` over 16; and that with the
//! place added (`scene::copy_matrix_with_place`) to `flt_5EA30C`.
//! One caller, `sub_4907E7`.
//! Manual patch with a private copy: the function begins with a call.
//! Self-test: 60 settings by lot on the game's matrices, put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `make_object_matrix_plain`, through
//! `rc2k_core::scene::matrices::make_object_matrix` (plain floats; the x87 version stays
//! for the Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::scene::matrix::{self};
use crate::screens::harness::Left;
use crate::service::floats::*;

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { matrix::make_object_matrix_plain() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
