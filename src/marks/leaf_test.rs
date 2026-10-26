//! The self-test of a small marks leaf on game variables: its inputs set by lot for each
//! case, the original and the Rust version compared on what it writes, and everything put
//! back afterwards.

use crate::framework::{Regs, SelfTest, game_write};
use crate::screens::harness::CallOriginal;
use crate::service::floats::{FLOAT, Floats, Tolerance, at_test_precisions, check_state_floats};
use crate::service::harness::{Region, Rust, at, capture, check_state, restore};

/// Runs `cases` cases: `inputs` (dwords) get floats by lot (some whole numbers, some 0,
/// negative or large), `written` is compared; `regs` are the entry registers.
pub fn test_leaf(
    t: &mut SelfTest,
    inputs: &[usize],
    written: &[usize],
    regs: Regs,
    cases: u32,
    call_original: CallOriginal,
    rust: Rust,
) {
    let regions: Vec<Region> = written.iter().map(|&a| at(a)).collect();
    let mut all: Vec<Region> = inputs.iter().map(|&a| at(a)).collect();
    all.extend(regions.iter().copied());
    let saved = capture(&all);
    for case in 0..cases {
        check_state(
            t,
            &format!("case {case}"),
            call_original,
            regs,
            &|| prepare_case(inputs, written, case),
            &regions,
            rust,
        );
    }
    restore(&all, &saved);
}

/// The same for a converted float leaf: each case at the tests' two precisions, the
/// written floats (and `float_regs`) held to `FLOAT` at extended precision.
// The leaf's whole description, as `check_state_floats` takes it.
#[allow(clippy::too_many_arguments)]
pub fn test_leaf_floats(
    t: &mut SelfTest,
    name: &'static str,
    inputs: &[usize],
    written: &[usize],
    regs: Regs,
    float_regs: &[(u8, Tolerance)],
    cases: u32,
    call_original: CallOriginal,
    rust: Rust,
) {
    let regions: Vec<Region> = written.iter().map(|&a| at(a)).collect();
    let mut all: Vec<Region> = inputs.iter().map(|&a| at(a)).collect();
    all.extend(regions.iter().copied());
    let saved = capture(&all);
    let places: Vec<(Region, Tolerance)> = regions.iter().map(|&r| (r, FLOAT)).collect();
    let tolerant = Floats {
        memory: &places,
        regs: float_regs,
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for case in 0..cases {
            check_state_floats(
                t,
                name,
                &format!("control word {word:#x}, case {case}"),
                call_original,
                regs,
                &|| prepare_case(inputs, written, case),
                &regions,
                held,
                rust,
            );
        }
    });
    restore(&all, &saved);
}

/// Case `case`'s inputs by lot and the written dwords filled with 0xAA.
fn prepare_case(inputs: &[usize], written: &[usize], case: u32) {
    for (n, &a) in inputs.iter().enumerate() {
        let k = case
            .wrapping_mul(0x9E37_79B9)
            .wrapping_add(n as u32 * 0x85EB_CA6B);
        let v = match (case + n as u32) % 5 {
            0 => (k % 2000) as f32 - 1000.0,
            1 => ((k >> 8) as f32) / 65536.0 - 128.0,
            2 => 0.0,
            3 => -(((k >> 4) % 100_000) as f32) * 1.5e-3,
            _ => (k as i32 >> 7) as f32,
        };
        unsafe { game_write(a, v.to_bits()) };
    }
    for &a in written {
        unsafe { game_write(a, 0xAAAA_AAAAu32) };
    }
}
