//! `marks::scale_axes` (0x4BFD73): the two axes at `flt_89CF1A` and `flt_89CF26` scaled: by `flt_89CECA` into `flt_89CF3E` / `flt_89CF4A`, by `flt_89CEEE` / `flt_89CEF2` and the per-part factors `flt_8BC1DE` .. into `flt_89CF6E` / `flt_89CF7A`, and the first two times `flt_8BC1F6` .. into `flt_89CF56` / `flt_89CF62`.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register after it)
//!
//! Callers: 0x4bf75e, 0x4c90b3. The original's instructions over the game's variables (register
//! forms as their bytes). Self-test: the original and the Rust version on test floats in
//! every variable it reads, all it writes compared; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `scale_axes_plain`, through
//! `rc2k_core::marks::scales::scale_axes` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::game_addr;

/// The Rust version (see the header).
pub unsafe fn scale_axes() {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x49CF1A]",
            "fmul dword ptr [{b} + 0x49CECA]",
            "fstp dword ptr [{b} + 0x49CF3E]",
            "fld dword ptr [{b} + 0x49CF1E]",
            "fmul dword ptr [{b} + 0x49CECA]",
            "fstp dword ptr [{b} + 0x49CF42]",
            "fld dword ptr [{b} + 0x49CF22]",
            "fmul dword ptr [{b} + 0x49CECA]",
            "fstp dword ptr [{b} + 0x49CF46]",
            "fld dword ptr [{b} + 0x49CF26]",
            "fmul dword ptr [{b} + 0x49CECA]",
            "fstp dword ptr [{b} + 0x49CF4A]",
            "fld dword ptr [{b} + 0x49CF2A]",
            "fmul dword ptr [{b} + 0x49CECA]",
            "fstp dword ptr [{b} + 0x49CF4E]",
            "fld dword ptr [{b} + 0x49CF2E]",
            "fmul dword ptr [{b} + 0x49CECA]",
            "fstp dword ptr [{b} + 0x49CF52]",
            "fld dword ptr [{b} + 0x49CF1A]",
            "fmul dword ptr [{b} + 0x49CEEE]",
            "fmul dword ptr [{b} + 0x4BC1DE]",
            "fstp dword ptr [{b} + 0x49CF6E]",
            "fld dword ptr [{b} + 0x49CF1E]",
            "fmul dword ptr [{b} + 0x49CEEE]",
            "fmul dword ptr [{b} + 0x4BC1E2]",
            "fstp dword ptr [{b} + 0x49CF72]",
            "fld dword ptr [{b} + 0x49CF22]",
            "fmul dword ptr [{b} + 0x49CEEE]",
            "fmul dword ptr [{b} + 0x4BC1E6]",
            "fstp dword ptr [{b} + 0x49CF76]",
            "fld dword ptr [{b} + 0x49CF26]",
            "fmul dword ptr [{b} + 0x49CEF2]",
            "fmul dword ptr [{b} + 0x4BC1EA]",
            "fstp dword ptr [{b} + 0x49CF7A]",
            "fld dword ptr [{b} + 0x49CF2A]",
            "fmul dword ptr [{b} + 0x49CEF2]",
            "fmul dword ptr [{b} + 0x4BC1EE]",
            "fstp dword ptr [{b} + 0x49CF7E]",
            "fld dword ptr [{b} + 0x49CF2E]",
            "fmul dword ptr [{b} + 0x49CEF2]",
            "fmul dword ptr [{b} + 0x4BC1F2]",
            "fstp dword ptr [{b} + 0x49CF82]",
            "fld dword ptr [{b} + 0x49CF3E]",
            "fmul dword ptr [{b} + 0x4BC1F6]",
            "fstp dword ptr [{b} + 0x49CF56]",
            "fld dword ptr [{b} + 0x49CF42]",
            "fmul dword ptr [{b} + 0x4BC1FA]",
            "fstp dword ptr [{b} + 0x49CF5A]",
            "fld dword ptr [{b} + 0x49CF46]",
            "fmul dword ptr [{b} + 0x4BC1FE]",
            "fstp dword ptr [{b} + 0x49CF5E]",
            "fld dword ptr [{b} + 0x49CF4A]",
            "fmul dword ptr [{b} + 0x4BC202]",
            "fstp dword ptr [{b} + 0x49CF62]",
            "fld dword ptr [{b} + 0x49CF4E]",
            "fmul dword ptr [{b} + 0x4BC206]",
            "fstp dword ptr [{b} + 0x49CF66]",
            "fld dword ptr [{b} + 0x49CF52]",
            "fmul dword ptr [{b} + 0x4BC20A]",
            "fstp dword ptr [{b} + 0x49CF6A]",
            b = in(reg) game_addr(0) as u32,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}
