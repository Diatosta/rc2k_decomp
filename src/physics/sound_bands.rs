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
use crate::race::owner_frame::Frame;
use crate::session::{Dword, Float};
use rc2k_core::physics::sound_bands::sound_band;

/// One twin: (address, value, factor, step).
pub type Twin = (usize, fn() -> &'static Float, fn() -> &'static Float, u32);

/// The exact x87 version on the value and the factor, the scratch keeps the number of `fistp`.
pub fn band_at(value: &Float, factor: &Float, scratch: &Dword, step: u32) -> u32 {
    let n = x87::mul_to_int(value.bits(), factor.bits()) as u32;
    scratch.set(n);
    let mut b = n.wrapping_sub(step);
    if (b as i32) < 0 {
        b = 0;
    }
    if (b.wrapping_sub(0x40) as i32) >= 0 {
        b = 0x40;
    }
    b
}

/// The band on the game's variables (the exact x87 version).
pub fn band_value(frame: &Frame, twin: Twin) -> u32 {
    band_at(twin.1(), twin.2(), &frame.sound_spots.band_scratch, twin.3)
}

/// The converted version (`rc2k_core::physics::sound_bands`, plain floats).
// Converted: `rc2k_core::physics::sound_bands::sound_band`.
pub fn band_at_plain(value: &Float, factor: &Float, scratch: &Dword, step: u32) -> u32 {
    let (n, b) = sound_band(value.get(), factor.get(), step);
    scratch.set(n as u32);
    b
}

/// The converted band on the game's variables.
pub fn band_value_plain(frame: &Frame, twin: Twin) -> u32 {
    band_at_plain(twin.1(), twin.2(), &frame.sound_spots.band_scratch, twin.3)
}
