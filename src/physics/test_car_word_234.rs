//! `physics::test_car_word_234` (0x43FDD5): `cmp` of `g_car` +0x234 with 0, then
//! `jz $+2` and `retn` either way: only the flags change (ZF when 0, CF clear). Its one
//! caller reads none of them (the next flag reader follows a `cmp`): by the look of it a
//! test whose work was cut out.
//!
//! In:  nothing
//! Out: every register kept; ZF = the word is 0, CF clear
//!
//! Caller: `sub_43B635` (0x43B96B), every physics step.
//! Self-test: a copy of the original (the word the test's) on 0, 1, -1, 0x80000000.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_read, rva};
use crate::physics::flag_result::FlagResult;

const WORD_RVA: usize = rva(0x71A20C); // g_car + 0x234

/// The core: the word's value, eax at entry.
pub fn test_word(word: u32, eax: u32) -> FlagResult {
    FlagResult {
        eax,
        ecx: None,
        ebx: None,
        carry: false,
        zero: Some(word == 0),
    }
}

/// The Rust version.
pub unsafe fn test_car_word_234(eax: u32) -> FlagResult {
    test_word(unsafe { game_read(WORD_RVA) }, eax)
}
