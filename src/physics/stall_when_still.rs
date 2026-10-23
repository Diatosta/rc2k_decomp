//! `physics::stall_when_still` (0x440B9D): the length of the motion kept by
//! `physics::keep_motion` (0x61409C .., `physics::vector_length`, rounded into
//! `dword_62400C`); when not 0 (C3 clear), `flt_6140AC` = the motion (0x71A17C ..) dot it
//! (`physics::dot_product`) over the length twice. Then, unless the length's size is
//! below `flt_613E40`, with `flt_6140AC` below `flt_613E44` and `dword_61416C` 0: gear 0
//! and both drive speeds 0. By the look of it the engine stalling when the car turns
//! back on itself (a reading).
//!
//! In:  nothing
//! Out: esi 0x71A17C and edi 0x61409C when the dot was taken, else esi 0x61409C; the rest
//!      kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B896), every physics step.
//! Self-test: a copy of the original on test variables (its two calls kept), 12 cases
//! under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EDI, ESI, Regs, game_addr, game_read, rva};
use crate::math::x87_run::{abs_compare, compare_full, ext_div2};
use crate::physics::dot_product::dot_product;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::rounded;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The kept motion (3), the scratch, the motion (3), the dot, the block, gear, speeds (2).
const VARIABLES: [usize; 12] = [
    0x61409C, 0x6140A0, 0x6140A4, 0x62400C, 0x71A17C, 0x71A180, 0x71A184, 0x6140AC, 0x61416C,
    0x71A1AC, 0x71A190, 0x71A194,
];

/// The Rust core on the variables at `v`; `r` as the original leaves it.
pub unsafe fn stall_at(v: &[u32], r: &mut Regs) {
    unsafe {
        let get = |k: usize| (v[k] as *const u32).read_unaligned();
        let put = |k: usize, x: u32| (v[k] as *mut u32).write_unaligned(x);
        r.esi = v[0];
        put(3, rounded(vector_length(v[0])));
        let zero: u32 = game_read(rva(0x5C5D90));
        if compare_full(get(3), zero) & 0x4000 == 0 {
            r.esi = v[4];
            r.edi = v[0];
            put(7, ext_div2(dot_product(v[4], v[0]), get(3), get(3)));
        }
        if abs_compare(get(3), game_read(rva(0x613E40))) & 0x100 != 0 {
            return;
        }
        if compare_full(get(7), game_read(rva(0x613E44))) & 0x100 == 0 {
            return;
        }
        if get(8) != 0 {
            return;
        }
        put(9, 0);
        put(10, 0);
        put(11, 0);
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version; `r` as the original leaves it.
pub unsafe fn stall_when_still(r: &mut Regs) {
    unsafe { stall_at(&game(), r) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { stall_when_still(&mut g) };
    Ok(Left::nothing().with(ESI, g.esi).with(EDI, g.edi))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
