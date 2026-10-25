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

use crate::math::x87;
use crate::race::sound_spots_store::RACE_SOUND_SPOTS;
use rc2k_core::physics::sound_bands::sound_band;

/// One twin: (address, value, factor, step).
pub type Twin = (usize, usize, usize, u32);

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The exact x87 version on the value, the factor and the scratch at these places (for
/// `physics::place_sound_e1`'s exact version).
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

/// The band on the game's variables (the exact x87 version).
pub unsafe fn band_value(twin: Twin) -> u32 {
    let g = |a: usize| crate::framework::game_addr(crate::framework::rva(a)) as u32;
    unsafe {
        band_at(
            g(twin.1),
            g(twin.2),
            RACE_SOUND_SPOTS.band_scratch.addr() as u32,
            twin.3,
        )
    }
}

/// The converted version (`rc2k_core::physics::sound_bands`, plain floats) on the
/// value, the factor and the scratch at these places.
// Converted: `rc2k_core::physics::sound_bands::sound_band`.
pub unsafe fn band_at_plain(value: u32, factor: u32, scratch: u32, step: u32) -> u32 {
    unsafe {
        let (n, b) = sound_band(f32::from_bits(at(value)), f32::from_bits(at(factor)), step);
        (scratch as *mut u32).write_unaligned(n as u32);
        b
    }
}

/// The converted band on the game's variables.
pub unsafe fn band_value_plain(twin: Twin) -> u32 {
    let g = |a: usize| crate::framework::game_addr(crate::framework::rva(a)) as u32;
    unsafe {
        band_at_plain(
            g(twin.1),
            g(twin.2),
            RACE_SOUND_SPOTS.band_scratch.addr() as u32,
            twin.3,
        )
    }
}
