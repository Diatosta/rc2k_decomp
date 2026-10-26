//! `scene::rotate_point` (0x44D110): the point in st(0) .. st(2) turned and placed by the
//! matrix at esi (rows 0x10 bytes apart, the place at +0x30), left there unrounded.
//!
//! In:  st(0), st(1), st(2) the point, esi the matrix
//! Out: st(0) .. st(2) the result (80 bits); every register kept
//!
//! Plain Rust since 2026-10-24: the x87's arithmetic in `X` (`math/x87_float.rs`), the point
//! and the result as 80-bit values (exact for what `X` holds). The original's instructions
//! are `asm_reference` in the tests. Refuses a null matrix.

use crate::scene::x87_mem::{fld, fld80, fstp80};

/// The Rust version: `p` as st(0) .. st(2), the result the same way.
pub unsafe fn rotate_point(m: u32, p: [[u8; 10]; 3]) -> [[u8; 10]; 3] {
    let mut out = [[0u8; 10]; 3];
    unsafe { rotate(p.as_ptr().cast(), m as *const u8, out.as_mut_ptr().cast()) };
    out
}

/// `fld tbyte` x3, the three rows' sums of products plus the place, `fstp tbyte` x3.
unsafe fn rotate(p: *const u8, m: *const u8, o: *mut u8) {
    let t0 = fld80(p, 0x14);
    let t1 = fld80(p, 0xA);
    let t2 = fld80(p, 0x0);
    let t3 = fld(m, 0x0);
    let t4 = t3 * t2;
    let t5 = fld(m, 0x10);
    let t6 = t5 * t1;
    let t7 = fld(m, 0x20);
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = fld(m, 0x30);
    let t12 = t10 + t11;
    let t13 = fld(m, 0x4);
    let t14 = t13 * t2;
    let t15 = fld(m, 0x14);
    let t16 = t15 * t1;
    let t17 = fld(m, 0x24);
    let t18 = t17 * t0;
    let t19 = t16 + t18;
    let t20 = t14 + t19;
    let t21 = fld(m, 0x34);
    let t22 = t20 + t21;
    let t23 = fld(m, 0x8);
    let t24 = t2 * t23;
    let t25 = fld(m, 0x18);
    let t26 = t1 * t25;
    let t27 = fld(m, 0x28);
    let t28 = t0 * t27;
    let t29 = t28 + t26;
    let t30 = t24 + t29;
    let t31 = fld(m, 0x38);
    let t32 = t30 + t31;
    fstp80(o, 0x0, t12);
    fstp80(o, 0xA, t22);
    fstp80(o, 0x14, t32);
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Kind, differential, put_t80};

    /// The original's instructions.
    unsafe fn rotate_ref(p: *mut u8, m: *mut u8, o: *mut u8) {
        unsafe {
            core::arch::asm!(
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
                "fstp tbyte ptr [{o}]", "fstp tbyte ptr [{o} + 10]", "fstp tbyte ptr [{o} + 20]",
                p = in(reg) p,
                m = in(reg) m,
                o = in(reg) o,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    #[test]
    fn rotate_matches_the_assembly() {
        // bytes 0 .. 30 the point, 32 .. 92 the matrix (15 dwords), 96 .. 126 the result
        differential::<32>(
            "rotate_point",
            6000,
            2,
            [Kind::Exact; 32],
            |rng, m| {
                m.fill(0xAAAA_AAAA);
                for k in 0..3 {
                    put_t80(m, 10 * k, rng.fl());
                }
                for d in m.iter_mut().skip(8).take(15) {
                    *d = rng.fl();
                }
            },
            |base| unsafe { rotate_ref(base, base.add(32), base.add(96)) },
            |base| unsafe { rotate(base, base.add(32), base.add(96)) },
        );
    }
}
