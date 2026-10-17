//! The self-test of a small marks leaf on game variables: its inputs set by lot for each
//! case, the original and the Rust version compared on what it writes, and everything put
//! back afterwards.

use crate::framework::{Regs, SelfTest, game_write};
use crate::screens::harness::CallOriginal;
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
        let prepare = || unsafe {
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
                game_write(a, v.to_bits());
            }
            for &a in written {
                game_write(a, 0xAAAA_AAAAu32);
            }
        };
        check_state(
            t,
            &format!("case {case}"),
            call_original,
            regs,
            &prepare,
            &regions,
            rust,
        );
    }
    restore(&all, &saved);
}
