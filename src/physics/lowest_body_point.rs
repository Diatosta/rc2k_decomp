//! `physics::lowest_body_point` (0x45D6DB): with more than four probe points
//! (`dword_71A238`), the deepest of the body's (points 4 .., their depths
//! `dword_6D2514[n]`, from 0x640000 down by the sign of the difference) into
//! `dword_6D3068` and its number into `dword_6D306C` (-1 for none); then `flt_6D3070` =
//! `dword_6D3068` / 65536 (with four or fewer, of what it held).
//!
//! In:  nothing
//! Out: eax the last depth looked at, ebx the count (when there are more than four); the
//!      rest kept; no flags read (callers go on with `mov`, `cmp`)
//!
//! Callers: `physics::feel_ground`, `sub_43BDD1` (twice), `sub_45EDB5`, `sub_45EFF3`.
//! Refuses (before anything) a count above 0x40 (the depths' room).
//! Self-test: `physics/copy_hooks_test.rs` (`test_moves_floats`: a copy on test
//! variables against the converted version), counts 0 .. 0x40; the refusal.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, rva};
use crate::math::x87;

/// The count, the lowest, its number, the share, then the 0x40 depths.
fn variables() -> Vec<usize> {
    let mut v = vec![0x71A238, 0x6D3068, 0x6D306C, 0x6D3070];
    v.extend((0..0x40).map(|k| 0x6D2514 + 4 * k));
    v
}

/// The Rust core on the variables at `v`. The registers it leaves (ebx the count, eax
/// the last depth) are read by none of its callers.
pub unsafe fn lowest_at(v: &[u32]) -> Result<(), Refused> {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        let n = get(0);
        if n > 0x40 {
            return Err(BadInput("more than 0x40 probe points (the depths' room)").into());
        }
        if n > 4 {
            put(1, 0x64_0000);
            put(2, u32::MAX);
            for k in 4..n {
                let depth = get(4 + k as usize);
                if (depth.wrapping_sub(get(1)) as i32) < 0 {
                    put(1, depth);
                    put(2, k);
                }
            }
        }
        put(3, x87::int_div(get(1) as i32, game_read(rva(0x5C5D98))));
        Ok(())
    }
}

fn game() -> Vec<u32> {
    variables()
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version (the exact one, for `physics::feel_ground`, `ground_contact`,
/// `find_clear_place`, `crash_contact`).
pub unsafe fn lowest_body_point() -> Result<(), Refused> {
    unsafe { lowest_at(&game()) }
}
