//! `physics::test_car_word` (0x44133F): eax = (`dword_719FE4` rol cl) & 0xFFFF, then
//! `cmp eax, 1`: CF when that is 0, ZF when it is 1. `dword_719FE4` is the work copy's
//! +0xC (`drive/car.rs`); what its bits are is not known yet.
//!
//! In:  cl the rotation
//! Out: eax the masked word; CF, ZF as said (callers read CF: `jnb`); the rest kept
//!
//! Callers (8): `sub_4410C8` and `sub_4411EC` (under `sub_4454D2`), four each, with cl
//! 5, 9, 0xE, ...
//! Shadow mode: `physics/flag_result.rs` (CF and ZF compared).
//! Self-test: a copy of the original on a test dword, rotations 0 .. 0x27 (the count is
//! taken mod 32) on several words.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::physics::flag_result::{FlagResult, flags_entry};

const WORD_RVA: usize = rva(0x719FE4); // dword_719FE4: g_car + 0xC

/// The Rust version on the word itself.
// Converted: `rc2k_core::physics::car_word::test_car_word`.
pub fn test_word(word: u32, cl: u8) -> FlagResult {
    let t = rc2k_core::physics::car_word::test_car_word(word, cl);
    FlagResult {
        eax: t.value,
        ecx: None,
        ebx: None,
        carry: t.carry,
        zero: Some(t.zero),
    }
}

/// The Rust version.
pub fn test_car_word(cl: u8) -> FlagResult {
    test_word(unsafe { game_read(WORD_RVA) }, cl)
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    Ok(test_car_word(r.ecx as u8))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    flags_entry(r, &rust)
}
