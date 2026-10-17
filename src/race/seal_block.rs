//! `race::seal_block` (0x402735): the block (`dword_4E0534` / `38`) enciphered with the
//! key (`dword_4E052C` / `30`) by the game's DES-like tables into `value` (0x4E053C) and
//! `dword_4E0540`, its sixteen subkeys left at `dword_4E04AC` ..; `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/save_seal.rs` (with `race::permute_bits`). Callers: `sub_40264D` and
//! `sub_41F0A3` (each `mov`s `value` after; no flags read). Shadow mode: the Rust version,
//! the subkeys and result put back, the original's, compared. Self-test: keys and blocks
//! by lot (0 and all ones among them), the game's put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::save_seal::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::seal_block", rva(0x402735), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { seal_block() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &seal_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = seal_regions();
    let inputs = seal_inputs();
    let mut kept = regions.clone();
    kept.extend(inputs.iter().copied());
    let saved = capture(&kept);
    let mut seed = 0x1357_2468u32;
    let mut cases = vec![
        [0u32; 4],
        [u32::MAX; 4],
        [0x0123_4567, 0x89AB_CDEF, 0xFEDC_BA98, 0x7654_3210],
    ];
    for _ in 0..8 {
        let mut c = [0u32; 4];
        for v in &mut c {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            *v = seed;
        }
        cases.push(c);
    }
    for c in &cases {
        let bytes: Vec<u8> = c.iter().flat_map(|v| v.to_le_bytes()).collect();
        check_state(
            &mut t,
            &format!("key {:x?}, block {:x?}", &c[..2], &c[2..]),
            call_original,
            Regs::SENTINEL,
            &|| {
                restore(&inputs, &bytes);
                for &(a, len) in &regions {
                    unsafe { std::ptr::write_bytes(a as *mut u8, 0x5A, len) };
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
