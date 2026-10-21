//! `physics::gear_bit` (0x43C5B7): eax = ecx + 1, CF = bit eax of `g_carGearBits`
//! (0x71A40C, the work copy's +0x434; `bt` on memory, so a bit string). The callers pass
//! a gear (-1 .. 7, so bits 0 .. 8) and test CF (`jb`/`jnb`): a gear that cannot be
//! taken, by the look of `sub_43C20D` (a reading).
//!
//! In:  ecx the gear
//! Out: eax = ecx + 1, CF the bit; ZF as before (no caller reads it); the rest kept
//!
//! Callers (10): the gearbox under `sub_4454D2` (`sub_43C20D`, `sub_43C2AE` twice,
//! `sub_43C58E`, `sub_43C5A0`, `sub_43C5C2` twice, `sub_43C8AC`, `sub_4651B5`, unowned
//! code at 0x43C9AE).
//! Refuses a bit outside 0 ..= 31 (past the dword; the original reads on along the copy).
//! Shadow mode: `physics/flag_result.rs`.
//! Self-test: a copy of the original on a test dword, every gear from -1 to 30 with
//! several bit patterns; the refusal.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::flag_result::{FlagResult, dispatch_flags};
use rc2k_core::physics::gearbox::GearBits;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::gear_bit", rva(0x43C5B7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const GEAR_BITS_RVA: usize = rva(0x71A40C); // g_carGearBits

/// The gear bits in the dword at `bits`.
pub unsafe fn bits_at(bits: u32) -> GearBits {
    GearBits(unsafe { (bits as *const u32).read_unaligned() })
}

/// The Rust version on the dword at `bits`.
// Converted: `rc2k_core::physics::gearbox::GearBits::test`.
pub unsafe fn gear_bit_at(bits: u32, gear: u32) -> Result<FlagResult, Refused> {
    let t = unsafe { bits_at(bits) }.test(gear)?;
    Ok(FlagResult {
        eax: t.bit,
        ecx: None,
        ebx: None,
        carry: t.barred,
        zero: None,
    })
}

/// The Rust version.
pub unsafe fn gear_bit(gear: u32) -> Result<FlagResult, Refused> {
    unsafe { gear_bit_at(game_addr(GEAR_BITS_RVA) as u32, gear) }
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    unsafe { gear_bit(r.ecx) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_flags(&HOOK, r, call_original, &rust);
}

/// A copy of the original reading its bits at `bits` (for this test and its callers').
pub fn test_copy(bits: u32) -> Result<crate::redirect::RedirectedCopy, String> {
    crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43C5B7,
            len: 0xB,
            entry: &[0x8B, 0xC1, 0x40, 0x0F, 0xA3, 0x05, 0x0C, 0xA4, 0x71, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x71A40C, bits)],
        1,
    )
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut word = Box::new(0u32);
    let at = &mut *word as *mut u32;
    let copy = match test_copy(at as u32) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    for pattern in [0u32, u32::MAX, 0x5555_5555, 0x0000_01A6, 0x8000_0001] {
        for gear in -1i32..=30 {
            unsafe { at.write_volatile(pattern) };
            let before = Regs {
                ecx: gear as u32,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let mine = unsafe { gear_bit_at(at as u32, gear as u32) };
            let ok = mine.as_ref().is_ok_and(|m| m.matches(&before, &ro));
            t.check(ok, || {
                format!(
                    "bits {pattern:#x}, gear {gear}: original eax {:#x} carry {}; rust {mine:?}",
                    ro.eax,
                    ro.carry()
                )
            });
        }
    }
    for gear in [31u32, 0x7FFF_FFFF, 0xFFFF_FFFE] {
        let refused = unsafe { gear_bit_at(at as u32, gear) }.is_err();
        t.check(refused, || format!("gear {gear:#x} not refused"));
    }
    drop(word);
    t.finish()
}
