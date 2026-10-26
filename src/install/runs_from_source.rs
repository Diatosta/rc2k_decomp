//! `install::runs_from_source` (0x412DA3): whether the game was started from the
//! directory the registry names as its installation's source.
//!
//! In:  nothing
//! Out: zero set if so; every register preserved
//! `read_registry`; then, only if the status is 1 and a source is named and the current
//! directory can be had (`files::current_directory` into `byte_94E8B0`), the two are
//! compared with `strings::same_text`. Everything else is "no" (zero and carry clear).
//! Callers: the main loop `sub_415383` on every start, and `install::check_clio_disc`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::install::outside::{Game, Outside, READ_REGISTRY};
use crate::install::read_registry::{SOURCE_RVA, STATUS_RVA};
use crate::install::stand_ins::stand_in;
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, capture, restore};
use crate::strings::same_text::compare_texts;
use std::cmp::Ordering;

/// byte_94E8B0: the scratch path (also `files::numbered_path`'s buffer).
pub const SCRATCH_PATH_RVA: usize = rva(0x94E8B0);

// ---- Rust version ----

/// None if the question does not arise, else how the source stands to the current
/// directory (`Equal`: started from there).
pub unsafe fn source_against_directory(
    outside: &mut dyn Outside,
) -> Result<Option<Ordering>, BadInput> {
    unsafe {
        outside.read_registry();
        if game_read::<u8>(STATUS_RVA) != 1 || game_read::<u8>(SOURCE_RVA) == 0 {
            return Ok(None);
        }
        let directory = game_addr(SCRATCH_PATH_RVA) as *mut u8;
        if !outside.current_directory(directory)? {
            return Ok(None);
        }
        Ok(Some(compare_texts(
            game_addr(SOURCE_RVA) as *const u8,
            directory,
        )))
    }
}

pub unsafe fn runs_from_source(outside: &mut dyn Outside) -> Result<bool, BadInput> {
    Ok(unsafe { source_against_directory(outside) }? == Some(Ordering::Equal))
}

fn regions() -> [Region; 1] {
    [(game_addr(SCRATCH_PATH_RVA) as u32, 0x100)]
}

/// (zero, carry) as the original leaves them.
fn flags(found: Option<Ordering>) -> (bool, bool) {
    (
        found == Some(Ordering::Equal),
        found == Some(Ordering::Less),
    )
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let regions = regions();
    let before = capture(&regions);
    let mine = match unsafe { source_against_directory(&mut Game) } {
        Ok(mine) => flags(mine),
        Err(e) => {
            restore(&regions, &before);
            return Err(Refused::from(e));
        }
    };
    r.set_zero(mine.0);
    r.set_carry(mine.1);
    Ok(())
}

const LEN: usize = 0x3D;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xE8, 0x6B, 0x0F, 0x00, 0x00, 0x80, 0x3D];
const CALLS: [usize; 3] = [0x01, 0x1E, 0x2D];
const READ_REGISTRY_RVA: usize = rva(0x413D14);

/// The original with its registry reading pointed at the test's world; the current
/// directory is this machine's. Also for `install::check_clio_disc`'s test.
pub unsafe fn copy_asking_the_test() -> Result<RedirectedCopy, &'static str> {
    unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x412DA3)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == READ_REGISTRY_RVA).then(|| stand_in(READ_REGISTRY)),
        )
    }
}
