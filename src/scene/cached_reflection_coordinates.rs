//! `scene::cached_reflection_coordinates` (0x451908): `scene::reflection_coordinates`
//! behind a table of results kept.
//!
//! In:  nothing; the two numbers in `dword_6399A8` and `dword_6399AC`
//! Out: the coordinates in `dword_6399B0`, `dword_6399B4`. With the table in use:
//!      ebx = the place in it, eax = the second coordinate, and after a working out
//!      edx and esi as that leaves them. Without it: as
//!      `scene::reflection_coordinates`
//!
//! With `dword_63D9B8` at 0 the original jumps straight on into the working out.
//! Else the normal's number & 0x3FF is a place in four tables of 0x400 dwords
//! (`dword_6399B8` the normal's number, `dword_63A9B8` the point's, `dword_63B9B8` and
//! `dword_63C9B8` the coordinates): if both numbers there are the ones asked for,
//! the coordinates come from the table; else they are worked out and all four kept.
//! One caller, `sub_45188C`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, EDX, ESI, Regs, game_read, game_write, rva};
use crate::scene::reflection_coordinates::{
    COORDINATES_RVA, NORMAL_NUMBER_RVA, POINT_NUMBER_RVA, reflection_coordinates,
};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- game data ----

const IN_USE_RVA: usize = rva(0x63D9B8); // dword_63D9B8
/// The four tables, 0x1000 bytes apart: the normal's number, the point's, the two
/// coordinates.
const TABLES_RVA: usize = rva(0x6399B8); // dword_6399B8, dword_63A9B8, dword_63B9B8, dword_63C9B8
const TABLE_BYTES: usize = 0x1000;

// ---- Rust version ----

// Converted: `rc2k_core::scene::reflection::{cache_place, cached}`
pub unsafe fn cached_reflection_coordinates() -> Result<Left, Refused> {
    use rc2k_core::scene::reflection::{cache_place, cached};
    unsafe {
        let worked_out = || {
            reflection_coordinates().map(|left| {
                Left::nothing()
                    .with(EAX, left.eax)
                    .with(EDX, left.edx)
                    .with(ESI, left.esi)
            })
        };
        if game_read::<u32>(IN_USE_RVA) == 0 {
            return Ok(worked_out()?);
        }
        let asked: [u32; 2] = [game_read(NORMAL_NUMBER_RVA), game_read(POINT_NUMBER_RVA)];
        let place = cache_place(asked[0]);
        let slot = |table: usize| TABLES_RVA + table * TABLE_BYTES + place * 4;
        if let Some(coordinates) = cached(asked, [game_read(slot(0)), game_read(slot(1))], || {
            [game_read(slot(2)), game_read(slot(3))]
        }) {
            game_write(COORDINATES_RVA, coordinates);
            return Ok(Left::nothing()
                .with(EBX, place as u32)
                .with(EAX, coordinates[1]));
        }
        let left = worked_out()?;
        let coordinates: [u32; 2] = game_read(COORDINATES_RVA);
        for (table, value) in [asked[0], asked[1], coordinates[0], coordinates[1]]
            .into_iter()
            .enumerate()
        {
            game_write(slot(table), value);
        }
        Ok(left.with(EBX, place as u32).with(EAX, coordinates[1]))
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { cached_reflection_coordinates() }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
