//! `physics::allowed_gear` (0x4651B5): eax kept or made 0. 0 when `g_car` +0x2B8
//! (`unk_71A290`) holds the bits of `flt_5C5D14` (100.0, compared as a dword); for eax
//! 1 or more (by the sign of eax - 1) also 0 when `physics::gear_bit` of it gives CF;
//! then 0 unless +0x3F4 (`unk_71A3CC`) is set. A reading: the gear asked for, unless the
//! gearbox refuses it.
//!
//! In:  eax the gear
//! Out: eax the gear or 0; the rest kept (ebx and ecx pushed around); no flags read
//!      (the caller stores eax)
//!
//! Caller: `sub_43C1E3` (0x43C1FD).
//! Refuses what `gear_bit` refuses (a bit past 31).
//! Self-test: a copy of the original (its two words the test's, its call to
//! `gear_bit`'s test copy on a test dword) against the core: gears -1 .. 9, the words'
//! settings, several bit patterns.

use crate::errors::Refused;
use crate::framework::{
    EAX, RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva,
};
use crate::physics::gear_bit::gear_bit_at;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::allowed_gear", rva(0x4651B5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The core: the word +0x2B8, the word +0x3F4 and the gear bits at these places.
pub unsafe fn allowed_gear_at(locked: u32, open: u32, bits: u32, eax: u32) -> Result<u32, Refused> {
    unsafe {
        if at(locked) == game_read::<u32>(rva(0x5C5D14)) {
            return Ok(0);
        }
        if (eax.wrapping_sub(1) as i32) >= 0 && gear_bit_at(bits, eax)?.carry {
            return Ok(0);
        }
        Ok(if at(open) != 0 { eax } else { 0 })
    }
}

/// The Rust version.
pub unsafe fn allowed_gear(eax: u32) -> Result<u32, Refused> {
    let g = |a: usize| game_addr(rva(a)) as u32;
    unsafe { allowed_gear_at(g(0x71A290), g(0x71A3CC), g(0x71A40C), eax) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { allowed_gear(r.eax) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 3];
    let base = cells.as_mut_ptr() as u32;
    let (locked, open, bits) = (base, base + 4, base + 8);
    let inner = match crate::physics::gear_bit::test_copy(bits) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let target = inner.addr();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4651B5,
            len: 0x31,
            entry: &[0x53, 0x8B, 0x1D, 0x90, 0xA2, 0x71, 0x00],
            calls: &[0x19],
            jumps_out: &[],
        },
        &|r| (r == rva(0x43C5B7)).then_some(target),
        &[(0x71A290, locked), (0x71A3CC, open)],
        2,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let hundred: u32 = unsafe { game_read(rva(0x5C5D14)) };
    for gear in [u32::MAX, 0, 1, 2, 3, 5, 7, 9] {
        for (l, o) in [(0u32, 1u32), (hundred, 1), (0, 0), (5, 3)] {
            for pattern in [0u32, 0x1FF, 0x0AA, 0x155] {
                let set = || unsafe {
                    (base as *mut [u32; 3]).write_volatile([l, o, pattern]);
                };
                set();
                let before = Regs {
                    eax: gear,
                    ..Regs::SENTINEL
                };
                let mut ro = before;
                unsafe { call_game(&mut ro, copy.addr()) };
                set();
                let mine = unsafe { allowed_gear_at(locked, open, bits, gear) };
                let ok = mine
                    .as_ref()
                    .is_ok_and(|&e| Left::nothing().with(EAX, e).matches(&before, &ro));
                t.check(ok, || {
                    format!("gear {gear:#x}, words {l:#x} {o}, bits {pattern:#x}: original eax {:#x}; rust {mine:?}", ro.eax)
                });
            }
        }
    }
    drop((copy, inner, cells));
    t.finish()
}
