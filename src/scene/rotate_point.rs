//! `scene::rotate_point` (0x44D110): the point in st(0) .. st(2) turned and placed by the
//! matrix at esi (rows 0x10 bytes apart, the place at +0x30), left there unrounded.
//!
//! In:  st(0), st(1), st(2) the point, esi the matrix
//! Out: st(0) .. st(2) the result (80 bits); every register kept
//!
//! The original's instructions, register operations as bytes. Refuses a null matrix.

use crate::math::x87::x87;

/// The Rust version: `p` as st(0) .. st(2), the result the same way.
pub unsafe fn rotate_point(m: u32, p: [[u8; 10]; 3]) -> [[u8; 10]; 3] {
    let mut out = [[0u8; 10]; 3];
    x87!(
        "fld tbyte ptr [{p} + 20]", "fld tbyte ptr [{p} + 10]", "fld tbyte ptr [{p}]",
        "fld dword ptr [{m}]", ".byte 0xD8, 0xC9",
        "fld dword ptr [{m} + 0x10]", ".byte 0xD8, 0xCB",
        "fld dword ptr [{m} + 0x20]", ".byte 0xD8, 0xCD",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fadd dword ptr [{m} + 0x30]", ".byte 0xD9, 0xCB",
        "fld dword ptr [{m} + 4]", ".byte 0xD8, 0xCA",
        "fld dword ptr [{m} + 0x14]", ".byte 0xD8, 0xCC",
        "fld dword ptr [{m} + 0x24]", ".byte 0xD8, 0xCB",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fadd dword ptr [{m} + 0x34]", ".byte 0xD9, 0xCB",
        "fld dword ptr [{m} + 8]", ".byte 0xDE, 0xCB",
        "fld dword ptr [{m} + 0x18]", ".byte 0xDE, 0xC9",
        "fld dword ptr [{m} + 0x28]", ".byte 0xDE, 0xCA",
        ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", "fadd dword ptr [{m} + 0x38]", ".byte 0xD9, 0xCA",
        "fstp tbyte ptr [{o}]", "fstp tbyte ptr [{o} + 10]", "fstp tbyte ptr [{o} + 20]";
        p = p.as_ptr(), m = m, o = out.as_mut_ptr()
    );
    out
}
