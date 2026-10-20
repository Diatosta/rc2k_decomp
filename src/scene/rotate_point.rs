//! `scene::rotate_point` (0x44D110): the point in st(0) .. st(2) turned and placed by the
//! matrix at esi (rows 0x10 bytes apart, the place at +0x30), left there unrounded.
//!
//! In:  st(0), st(1), st(2) the point, esi the matrix
//! Out: st(0) .. st(2) the result (80 bits); every register kept
//!
//! The original's instructions, register operations as bytes. Thunk by `triple_hook!`.
//! 87 callers (the marks and the race's drawing). Refuses a null matrix. Self-test: the
//! original and the Rust version on matrices and points by lot (some points unrounded).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, rva};
use crate::math::x87::x87;
use crate::race::stage_prefs::left_of;
use crate::scene::vector3::{Triple80, triple_hook};
use crate::screens::harness::Left;
use crate::service::floats::*;
use std::cell::Cell;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static TRIPLE: Triple80 = Triple80::new();

pub static HOOK: RawHook =
    RawHook::new("scene::rotate_point", rva(0x44D110), thunk, &ORIGINAL).self_test(self_test);

triple_hook!(
    thunk,
    call_original,
    original_with_triple,
    dispatch = dispatch,
    original = ORIGINAL,
    triple = TRIPLE
);

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

/// The converted version (`rc2k_core::vector`, plain floats): the hook's.
pub unsafe fn rotate_point_plain(m: u32, p: [[u8; 10]; 3]) -> [[u8; 10]; 3] {
    use rc2k_core::extended::{from_f64, to_f64};
    let m = unsafe { crate::scene::vector3::read_floats(m) };
    rc2k_core::vector::rotate_point(&m, p.map(to_f64)).map(from_f64)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.esi == 0 {
        return Err(BadInput("a null matrix").into());
    }
    TRIPLE.set_outputs(unsafe { rotate_point_plain(r.esi, TRIPLE.inputs()) });
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let floats = Floats {
        memory: &[(TRIPLE.output_region(), EXTENDED)],
        regs: &[],
    };
    dispatch_state_floats(
        &HOOK,
        r,
        original_with_triple,
        &[TRIPLE.output_region()],
        &floats,
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    use crate::scene::matrix3::{extended, test_floats};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let matrix: Cell<[u32; 16]> = Cell::new([0; 16]);
    let tolerant = Floats {
        memory: &[(TRIPLE.output_region(), EXTENDED)],
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, floats| {
        for seed in 1..=40u32 {
            let mut m = [0u32; 16];
            test_floats(seed * 7 + 3, &mut m);
            let mut f = [0u32; 6];
            test_floats(seed * 11 + 5, &mut f);
            // Every other case unrounded: a float less another, kept at 80 bits.
            let point: [[u8; 10]; 3] = std::array::from_fn(|n| {
                if seed % 2 == 0 {
                    extended(f[n])
                } else {
                    crate::math::x87::sub_80(f[n], f[n + 3])
                }
            });
            check_state_floats(
                &mut t,
                "scene::rotate_point",
                &format!("control word {word:#x}, seed {seed}: matrix {m:x?}, point {point:x?}"),
                original_with_triple,
                Regs {
                    esi: matrix.as_ptr() as u32,
                    ..Regs::SENTINEL
                },
                &|| {
                    matrix.set(m);
                    TRIPLE.set_inputs(point);
                    TRIPLE.set_outputs([[0xAA; 10]; 3]);
                },
                &[TRIPLE.output_region()],
                floats,
                &rust,
            );
        }
    });
    t.finish()
}
