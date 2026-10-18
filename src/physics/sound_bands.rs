//! Four twins (IDA 2026-10-18) that make a band 0 ..= 0x40 out of a float: ebx =
//! `fistp` of value x factor (through `dword_6D1AEC`), less a step, held to 0 ..= 0x40
//! (by the sign, `jns` / `js`):
//! - `physics::sound_band_1` (0x45D067): `flt_6D3108` x `flt_6D1AE0`, less 1;
//! - `physics::sound_band_2` (0x45D097): `g_car` +0x1C4 x `flt_6D1AE0`, less 10;
//! - `physics::sound_band_3` (0x45D0F1): `flt_6D3CD0` x `flt_6D1AE8`, less 10;
//! - `physics::sound_band_4` (0x45D121): `g_car` +0x1C4 x `flt_6D1AE4`, less 1.
//!
//! A reading: volume or pitch steps for the car's sounds (their callers `sub_45CD60` ..
//! `sub_45CF95` drive the sound objects).
//!
//! In:  nothing
//! Out: ebx the band; the rest kept; no flags read (the callers compare ebx next)
//!
//! Self-test: a copy of each original (its value, factor and `dword_6D1AEC` the test's)
//! against the core: values by lot about the band's ends, under three precisions.

use crate::framework::{EBX, RawHook, Regs, SelfTest, call_game};
use crate::math::x87;
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::harness::capture;

pub const SCRATCH: usize = 0x6D1AEC; // dword_6D1AEC

/// One twin: (address, value, factor, step).
pub type Twin = (usize, usize, usize, u32);

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The core: the value, the factor and the scratch at these places.
pub unsafe fn band_at(value: u32, factor: u32, scratch: u32, step: u32) -> u32 {
    unsafe {
        let n = x87::mul_to_int(at(value), at(factor)) as u32;
        (scratch as *mut u32).write_unaligned(n);
        let mut b = n.wrapping_sub(step);
        if (b as i32) < 0 {
            b = 0;
        }
        if (b.wrapping_sub(0x40) as i32) >= 0 {
            b = 0x40;
        }
        b
    }
}

/// The band on the game's variables.
pub unsafe fn band_value(twin: Twin) -> u32 {
    let g = |a: usize| crate::framework::game_addr(crate::framework::rva(a)) as u32;
    unsafe { band_at(g(twin.1), g(twin.2), g(SCRATCH), twin.3) }
}

/// On the game's variables.
pub unsafe fn band(twin: Twin) -> Left {
    let g = |a: usize| crate::framework::game_addr(crate::framework::rva(a)) as u32;
    let b = unsafe { band_at(g(twin.1), g(twin.2), g(SCRATCH), twin.3) };
    Left::nothing().with(EBX, b)
}

/// The test of one twin.
pub fn test_twin(hook: &RawHook, twin: Twin, entry: &[u8]) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 3];
    let base = cells.as_mut_ptr() as u32;
    let (value, factor, scratch) = (base, base + 4, base + 8);
    let copy = match crate::audio::test_copy::make(
        hook,
        &crate::audio::test_copy::Copy {
            address: twin.0,
            len: 0x30,
            entry,
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(twin.1, value), (twin.2, factor), (SCRATCH, scratch)],
        4,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 12)];
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..12u32 {
                let fill = || {
                    let mut f = [0u32; 1];
                    test_floats(seed * 13 + 5, &mut f);
                    let v = match seed {
                        0 => 0x7FC0_0000,
                        1 => (-3.0f32).to_bits(),
                        2 => 1.0e12f32.to_bits(),
                        _ => (f32::from_bits(f[0]) % 90.0).to_bits(),
                    };
                    unsafe { (base as *mut [u32; 3]).write([v, 1.0f32.to_bits(), 0xAAAA_AAAA]) };
                };
                fill();
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = capture(&all);
                fill();
                let b = unsafe { band_at(value, factor, scratch, twin.3) };
                let mine = capture(&all);
                let regs_ok = Left::nothing().with(EBX, b).matches(&Regs::SENTINEL, &ro);
                t.check(regs_ok && theirs == mine, || {
                    format!("control word {word:#x}, seed {seed}: original ebx {:#x} {theirs:x?}; rust {b:#x} {mine:x?}", ro.ebx)
                });
            }
        });
    }
    drop((copy, cells));
    t.finish()
}
