//! The car specification's units (0x401CBA, 0x401C42, 0x401CF1, 0x401D28): a car's
//! figure, stored in imperial units, written in the units the game is set to
//! (`g_isImperialUnits`, 0x51DE2C: 0 imperial, 1 metric), with its label.
//!
//! The metric ones go through a conversion of the same shape as `math::miles_to_km`:
//! (v * factor + 5000) / 10000 with a 64-bit product, the factor 7460 for power
//! (`sub_40196F`, bhp to kW), 9144 for lengths (`sub_401939`, to mm), 4536 for weight
//! (`sub_401954`, lb to kg), 1383 for torque (`sub_40198A`, lb.ft to kg.m); all four
//! inlined. The figures are tenths.
//!
//! The originals each take eax = the figure, ebx = the cursor; ebx moved on; everything else
//! preserved (`push`/`pop` around). Their hooks were removed 2026-10-21 (conversion phase 4):
//! the only caller left is `screens::draw_car_figures`, which calls these directly.

use crate::errors::Refused;
use crate::framework::rva;
use crate::strings::units::{Units, figure_bytes, units};

pub const UNITS_RVA: usize = rva(0x51DE2C); // g_isImperialUnits

/// The setting, refused when it is neither of the two labels' places.
fn setting() -> Result<Units, Refused> {
    Ok(crate::strings::units::setting(units())?)
}

/// `strings::append_power` (0x401CBA). Converted: `rc2k_core::math::units::power`.
pub fn append_power(value: u32) -> Result<Vec<u8>, Refused> {
    Ok(figure_bytes(rc2k_core::math::units::power(
        value,
        setting()?,
    )))
}

/// `strings::append_length` (0x401C42). Converted: `rc2k_core::math::units::length`; the
/// not-known figure needs no setting.
pub fn append_length(value: u32) -> Result<Vec<u8>, Refused> {
    let units = if (value >> 8) & 0xFF == 0xFF {
        Units::Imperial
    } else {
        setting()?
    };
    Ok(figure_bytes(rc2k_core::math::units::length(value, units)))
}

/// `strings::append_weight` (0x401CF1). Converted: `rc2k_core::math::units::weight`.
pub fn append_weight(value: u32) -> Result<Vec<u8>, Refused> {
    Ok(figure_bytes(rc2k_core::math::units::weight(
        value,
        setting()?,
    )))
}

/// `strings::append_torque` (0x401D28). Converted: `rc2k_core::math::units::torque`.
pub fn append_torque(value: u32) -> Result<Vec<u8>, Refused> {
    Ok(figure_bytes(rc2k_core::math::units::torque(
        value,
        setting()?,
    )))
}
