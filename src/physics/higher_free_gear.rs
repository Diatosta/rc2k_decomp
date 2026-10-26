//! `physics::higher_free_gear` (0x43C5A0): from the gear in ebx up, the first higher gear
//! whose `g_carGearBits` bit (`physics::gear_bit`) is clear: ecx = ebx + 1, + 2, .. up to
//! `g_gearCount` (0x614C10, unsigned); CF clear with ecx that gear, or CF set with ecx one
//! past the count when none is.
//!
//! In:  ebx the gear
//! Out: ecx, eax `gear_bit`'s (ecx + 1 of the last tested), CF; ZF not read by the
//!      caller (`jb`); the rest kept
//!
//! A loop target (0x43C5A3) is inside the first five bytes: `.private_copy`, its call
//! relocated. Caller: `sub_43C2AE` (0x43C365), the gearbox under `sub_4454D2`.
//! Refuses what `gear_bit` refuses (a bit past the dword: a count of 31 or more lets the
//! search run there), nothing written.
//! Shadow mode: `physics/flag_result.rs`.
//! Self-test: a copy of the original on a test count whose call goes to a copy of
//! `gear_bit` on a test dword, gears -1 .. 8, counts 4 .. 7 on several bit patterns;
//! refusals.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::flag_result::{FlagResult, dispatch_flags};
use crate::physics::gear_bit::{GEAR_BITS_RVA, bits_at};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::higher_free_gear", rva(0x43C5A0), thunk, &ORIGINAL)
        .private_copy(0x17)
        .relocate_calls(&[3])
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const GEAR_COUNT_RVA: usize = rva(0x614C10); // g_gearCount: forward gears

/// The Rust version on the bits at `bits` and the count `count`.
// Converted: `rc2k_core::physics::gearbox::GearBits::higher_free`.
pub unsafe fn higher_free_gear_at(bits: u32, count: u32, gear: u32) -> Result<FlagResult, Refused> {
    let s = unsafe { bits_at(bits) }.higher_free(gear, count)?;
    Ok(FlagResult {
        eax: s.bit,
        ecx: Some(s.gear),
        ebx: None,
        carry: !s.found,
        zero: None,
    })
}

/// The Rust version.
pub unsafe fn higher_free_gear(gear: u32) -> Result<FlagResult, Refused> {
    unsafe {
        higher_free_gear_at(
            game_addr(GEAR_BITS_RVA) as u32,
            crate::framework::game_read(GEAR_COUNT_RVA),
            gear,
        )
    }
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    unsafe { higher_free_gear(r.ebx) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_flags(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = Box::new([0u32; 2]);
    let bits = cells.as_mut_ptr();
    let count = unsafe { bits.add(1) };
    let bit_copy = match crate::physics::gear_bit::test_copy(bits as u32) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let target = bit_copy.addr();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43C5A0,
            len: 0x17,
            entry: &[0x8B, 0xCB, 0x41, 0xE8, 0x0F, 0x00, 0x00, 0x00],
            calls: &[3],
            jumps_out: &[],
        },
        &|r| (r == rva(0x43C5B7)).then_some(target),
        &[(0x614C10, count as u32)],
        1,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    for pattern in [0u32, 0x3FE, 0x0F0, 0x5555_5555, 0x0000_01A6, 0xFFFF_FFFF] {
        for n in 4u32..=7 {
            for gear in -1i32..=8 {
                unsafe {
                    bits.write_volatile(pattern);
                    count.write_volatile(n);
                }
                let before = Regs {
                    ebx: gear as u32,
                    ..Regs::SENTINEL
                };
                let mut ro = before;
                unsafe { call_game(&mut ro, copy.addr()) };
                let mine = unsafe { higher_free_gear_at(bits as u32, n, gear as u32) };
                let ok = mine.as_ref().is_ok_and(|m| m.matches(&before, &ro));
                t.check(ok, || {
                    format!(
                        "bits {pattern:#x}, count {n}, gear {gear}: original eax {:#x} ecx {:#x} carry {}; rust {mine:?}",
                        ro.eax,
                        ro.ecx,
                        ro.carry()
                    )
                });
            }
        }
    }
    // A count of 31 or more with every bit set runs past the dword.
    unsafe { bits.write_volatile(u32::MAX) };
    for (n, gear) in [(31u32, 1u32), (u32::MAX, 5), (7, 31)] {
        let refused = unsafe { higher_free_gear_at(bits as u32, n, gear) }.is_err();
        t.check(refused, || format!("count {n:#x}, gear {gear} not refused"));
    }
    drop((copy, bit_copy, cells));
    t.finish()
}
