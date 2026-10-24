//! `marks::scale_axes` (0x4BFD73): the two axes at `flt_89CF1A` and `flt_89CF26` scaled: by `flt_89CECA` into `flt_89CF3E` / `flt_89CF4A`, by `flt_89CEEE` / `flt_89CEF2` and the per-part factors `flt_8BC1DE` .. into `flt_89CF6E` / `flt_89CF7A`, and the first two times `flt_8BC1F6` .. into `flt_89CF56` / `flt_89CF62`.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register after it)
//!
//! Callers: 0x4bf75e, 0x4c90b3. The original's instructions over the game's variables
//! (register forms as their bytes).
//!
//! the x87 version stays for the Rust callers in unconverted code).

/// The Rust version (see the header).
pub unsafe fn scale_axes() {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{mk} + 0x137FDA]",
            "fmul dword ptr [{mk} + 0x137F8A]",
            "fstp dword ptr [{mk} + 0x137FFE]",
            "fld dword ptr [{mk} + 0x137FDE]",
            "fmul dword ptr [{mk} + 0x137F8A]",
            "fstp dword ptr [{mk} + 0x138002]",
            "fld dword ptr [{mk} + 0x137FE2]",
            "fmul dword ptr [{mk} + 0x137F8A]",
            "fstp dword ptr [{mk} + 0x138006]",
            "fld dword ptr [{mk} + 0x137FE6]",
            "fmul dword ptr [{mk} + 0x137F8A]",
            "fstp dword ptr [{mk} + 0x13800A]",
            "fld dword ptr [{mk} + 0x137FEA]",
            "fmul dword ptr [{mk} + 0x137F8A]",
            "fstp dword ptr [{mk} + 0x13800E]",
            "fld dword ptr [{mk} + 0x137FEE]",
            "fmul dword ptr [{mk} + 0x137F8A]",
            "fstp dword ptr [{mk} + 0x138012]",
            "fld dword ptr [{mk} + 0x137FDA]",
            "fmul dword ptr [{mk} + 0x137FAE]",
            "fmul dword ptr [{mk} + 0x15729E]",
            "fstp dword ptr [{mk} + 0x13802E]",
            "fld dword ptr [{mk} + 0x137FDE]",
            "fmul dword ptr [{mk} + 0x137FAE]",
            "fmul dword ptr [{mk} + 0x1572A2]",
            "fstp dword ptr [{mk} + 0x138032]",
            "fld dword ptr [{mk} + 0x137FE2]",
            "fmul dword ptr [{mk} + 0x137FAE]",
            "fmul dword ptr [{mk} + 0x1572A6]",
            "fstp dword ptr [{mk} + 0x138036]",
            "fld dword ptr [{mk} + 0x137FE6]",
            "fmul dword ptr [{mk} + 0x137FB2]",
            "fmul dword ptr [{mk} + 0x1572AA]",
            "fstp dword ptr [{mk} + 0x13803A]",
            "fld dword ptr [{mk} + 0x137FEA]",
            "fmul dword ptr [{mk} + 0x137FB2]",
            "fmul dword ptr [{mk} + 0x1572AE]",
            "fstp dword ptr [{mk} + 0x13803E]",
            "fld dword ptr [{mk} + 0x137FEE]",
            "fmul dword ptr [{mk} + 0x137FB2]",
            "fmul dword ptr [{mk} + 0x1572B2]",
            "fstp dword ptr [{mk} + 0x138042]",
            "fld dword ptr [{mk} + 0x137FFE]",
            "fmul dword ptr [{mk} + 0x1572B6]",
            "fstp dword ptr [{mk} + 0x138016]",
            "fld dword ptr [{mk} + 0x138002]",
            "fmul dword ptr [{mk} + 0x1572BA]",
            "fstp dword ptr [{mk} + 0x13801A]",
            "fld dword ptr [{mk} + 0x138006]",
            "fmul dword ptr [{mk} + 0x1572BE]",
            "fstp dword ptr [{mk} + 0x13801E]",
            "fld dword ptr [{mk} + 0x13800A]",
            "fmul dword ptr [{mk} + 0x1572C2]",
            "fstp dword ptr [{mk} + 0x138022]",
            "fld dword ptr [{mk} + 0x13800E]",
            "fmul dword ptr [{mk} + 0x1572C6]",
            "fstp dword ptr [{mk} + 0x138026]",
            "fld dword ptr [{mk} + 0x138012]",
            "fmul dword ptr [{mk} + 0x1572CA]",
            "fstp dword ptr [{mk} + 0x13802A]",
            mk = in(reg) crate::marks::store::base(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}
