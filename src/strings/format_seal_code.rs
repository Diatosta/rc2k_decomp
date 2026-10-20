//! `strings::format_seal_code` (0x4026C9): two dwords as "AABB-CCDD-EEFF-GGHH" in
//! `byte_4E0544`: each dword's bytes from the lowest, two hex digits each
//! (`strings::hex_append`), a '-' after every second byte; no NUL (the buffer keeps whatever
//! follows). The dwords are kept in `value` / `dword_4E0540`. (Stream A, part F1: the stage
//! results page shows a seal code with it.)
//!
//! In:  eax, ebx = the two dwords
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `screens::draw_stage_results` (0x48E87D). Shadow mode: by state. Self-test:
//! values by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("strings::format_seal_code", rva(0x4026C9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The 19 characters.
pub fn seal_code(a: u32, b: u32) -> Vec<u8> {
    rc2k_core::strings::format::seal_code(a, b)
}

/// The Rust version on the game's variables.
pub unsafe fn format_seal_code(a: u32, b: u32) {
    unsafe {
        game_write(rva(0x4E053C), a);
        game_write(rva(0x4E0540), b);
        let code = seal_code(a, b);
        std::ptr::copy_nonoverlapping(
            code.as_ptr(),
            game_addr(rva(0x4E0544)) as *mut u8,
            code.len(),
        );
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { format_seal_code(r.eax, r.ebx) };
    Ok(Left::nothing())
}

fn regions() -> [Region; 1] {
    [(game_addr(rva(0x4E053C)) as u32, 0x20)]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    let mut seed = 0x1234_5678u32;
    for _ in 0..8 {
        seed = seed.wrapping_mul(0x0101_9E35).wrapping_add(0x3C6E_F372);
        let (a, b) = (seed, seed.rotate_left(13) ^ 0xA5A5_0F0F);
        check_state(
            &mut t,
            &format!("{a:#x} {b:#x}"),
            call_original,
            Regs {
                eax: a,
                ebx: b,
                ..Regs::SENTINEL
            },
            &|| {},
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
