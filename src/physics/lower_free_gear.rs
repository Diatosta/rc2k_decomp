//! `physics::lower_free_gear` (0x43C58E): from the gear in ebx down, the first lower gear
//! whose `g_carGearBits` bit (`physics::gear_bit`) is clear: ecx = ebx - 1, - 2, ..; CF
//! clear with ecx that gear, or CF set when ecx reaches 0 (neutral) first.
//!
//! In:  ebx the gear
//! Out: ecx the gear found (or 0), eax `gear_bit`'s (ecx + 1; kept when none was
//!      tested), CF; ZF not read by the callers (`jb`); the rest kept
//!
//! A loop target (0x43C590) is inside the first five bytes: `.private_copy`, its call
//! relocated. Callers (2): `sub_43C2AE` (0x43C397), `sub_43C521` (in `sub_43C2AE`), the
//! gearbox under `sub_4454D2`.
//! Refuses what `gear_bit` refuses (a bit past the dword: from ebx 0 or above 32 the
//! search would run there), nothing written.
//! Shadow mode: `physics/flag_result.rs`.
//! Self-test: a copy of the original whose call goes to a copy of `gear_bit` on a test
//! dword, gears 1 .. 9 on several bit patterns; refusals.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::flag_result::{FlagResult, dispatch_flags};
use crate::physics::gear_bit::{GEAR_BITS_RVA, gear_bit_at};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::lower_free_gear", rva(0x43C58E), thunk, &ORIGINAL)
        .private_copy(0x12)
        .relocate_calls(&[8])
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust core on the bits at `bits`; `eax` what eax held at entry.
pub unsafe fn lower_free_gear_at(bits: u32, gear: u32, eax: u32) -> Result<FlagResult, Refused> {
    let mut ecx = gear;
    let mut eax = eax;
    loop {
        ecx = ecx.wrapping_sub(1);
        if ecx == 0 {
            break Ok(FlagResult {
                eax,
                ecx: Some(0),
                ebx: None,
                carry: true,
                zero: None,
            });
        }
        let g = unsafe { gear_bit_at(bits, ecx) }?;
        eax = g.eax;
        if !g.carry {
            break Ok(FlagResult {
                eax,
                ecx: Some(ecx),
                ebx: None,
                carry: false,
                zero: None,
            });
        }
    }
}

/// The Rust version.
pub unsafe fn lower_free_gear(gear: u32, eax: u32) -> Result<FlagResult, Refused> {
    unsafe { lower_free_gear_at(game_addr(GEAR_BITS_RVA) as u32, gear, eax) }
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    unsafe { lower_free_gear(r.ebx, r.eax) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_flags(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut word = Box::new(0u32);
    let at = &mut *word as *mut u32;
    let bit_copy = match crate::physics::gear_bit::test_copy(at as u32) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let target = bit_copy.addr();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43C58E,
            len: 0x12,
            entry: &[0x8B, 0xCB, 0x49, 0x83, 0xF9, 0x00],
            calls: &[8],
            jumps_out: &[],
        },
        &|r| (r == rva(0x43C5B7)).then_some(target),
        &[],
        0,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    for pattern in [0u32, 0x3FE, 0x0F0, 0x5555_5555, 0x0000_01A6, 0xAAAA_AAAA] {
        for gear in 1u32..=9 {
            unsafe { at.write_volatile(pattern) };
            let before = Regs {
                ebx: gear,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let mine = unsafe { lower_free_gear_at(at as u32, gear, before.eax) };
            let ok = mine.as_ref().is_ok_and(|m| m.matches(&before, &ro));
            t.check(ok, || {
                format!(
                    "bits {pattern:#x}, gear {gear}: original eax {:#x} ecx {:#x} carry {}; rust {mine:?}",
                    ro.eax,
                    ro.ecx,
                    ro.carry()
                )
            });
        }
    }
    // From 0 the search runs down past -1, from 33 it starts past the dword.
    unsafe { at.write_volatile(u32::MAX) };
    for gear in [0u32, 33] {
        let refused = unsafe { lower_free_gear_at(at as u32, gear, 0) }.is_err();
        t.check(refused, || format!("gear {gear} not refused"));
    }
    drop((copy, bit_copy, word));
    t.finish()
}
