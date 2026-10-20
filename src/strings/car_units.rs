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
//! Each formatter: eax = the figure, ebx = the cursor; ebx moved on; everything else
//! preserved (`push`/`pop` around).

use crate::errors::Refused;
use crate::framework::{EBX, RawHook, Regs, SelfTest, game_write, rva};
use crate::strings::units::{Units, figure_bytes, units};
use crate::strings::utils::bytes_str;

pub const UNITS_RVA: usize = rva(0x51DE2C); // g_isImperialUnits

/// The setting, refused when it is neither of the two labels' places.
fn setting() -> Result<Units, Refused> {
    Ok(crate::strings::units::setting(units())?)
}

/// `strings::append_power` (0x401CBA). Converted: `rc2k_core::units::power`.
pub fn append_power(value: u32) -> Result<Vec<u8>, Refused> {
    Ok(figure_bytes(rc2k_core::units::power(value, setting()?)))
}

/// `strings::append_length` (0x401C42). Converted: `rc2k_core::units::length`; the
/// not-known figure needs no setting.
pub fn append_length(value: u32) -> Result<Vec<u8>, Refused> {
    let units = if (value >> 8) & 0xFF == 0xFF {
        Units::Imperial
    } else {
        setting()?
    };
    Ok(figure_bytes(rc2k_core::units::length(value, units)))
}

/// `strings::append_weight` (0x401CF1). Converted: `rc2k_core::units::weight`.
pub fn append_weight(value: u32) -> Result<Vec<u8>, Refused> {
    Ok(figure_bytes(rc2k_core::units::weight(value, setting()?)))
}

/// `strings::append_torque` (0x401D28). Converted: `rc2k_core::units::torque`.
pub fn append_torque(value: u32) -> Result<Vec<u8>, Refused> {
    Ok(figure_bytes(rc2k_core::units::torque(value, setting()?)))
}

/// A formatter's `dispatch`: written at the cursor, the cursor moved on.
pub fn dispatch_unit(
    hook: &RawHook,
    r: &mut Regs,
    call_original: unsafe extern "C" fn(*mut Regs),
    format: fn(u32) -> Result<Vec<u8>, Refused>,
) {
    hook.count_call();
    match format(r.eax) {
        Ok(text) => {
            if crate::framework::shadow() {
                let mut o = *r;
                unsafe { call_original(&mut o) };
                let theirs = unsafe {
                    std::slice::from_raw_parts(
                        r.ebx as *const u8,
                        (o.ebx as usize).wrapping_sub(r.ebx as usize).min(0x100),
                    )
                };
                if theirs != &text[..] || !o.preserved_except(r, EBX) {
                    hook.mismatch(|| {
                        format!(
                            "value {:#x}, units {}: original {}; rust {}",
                            r.eax,
                            units(),
                            bytes_str(theirs),
                            bytes_str(&text)
                        )
                    });
                }
                *r = Regs { esp: r.esp, ..o };
                return;
            }
            unsafe {
                std::ptr::copy_nonoverlapping(text.as_ptr(), r.ebx as *mut u8, text.len());
            }
            r.ebx += text.len() as u32;
        }
        Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
    }
}

/// A formatter's self-test: both settings, figures by lot and at the ends.
pub fn check_unit(
    t: &mut SelfTest,
    call_original: unsafe extern "C" fn(*mut Regs),
    format: fn(u32) -> Result<Vec<u8>, Refused>,
) {
    let saved = units();
    let mut values: Vec<u32> = (0..300).chain([0xFE_FF, 0xFF_00, 0x1_FF_00]).collect();
    let mut seed = 0x0CA2_u32;
    for _ in 0..200 {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        values.push(seed >> (seed % 24));
    }
    values.extend([u32::MAX, 0x8000_0000, 0x7FFF_FFFF]);
    for setting in [0u32, 1] {
        unsafe { game_write(UNITS_RVA, setting) };
        for &v in &values {
            let mine = format(v).unwrap_or_default();
            let mut buf = vec![0xAAu8; mine.len() + 0x100];
            let before = Regs {
                eax: v,
                ebx: buf.as_mut_ptr() as u32,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let length = (ro.ebx as usize).wrapping_sub(buf.as_ptr() as usize);
            let preserved = ro.preserved_except(&before, EBX);
            t.check(buf.get(..length) == Some(&mine[..]) && preserved, || {
                format!(
                    "value {v:#x}, units {setting}: original {} rust {} preserved={preserved}",
                    bytes_str(buf.get(..length).unwrap_or(&[])),
                    bytes_str(&mine)
                )
            });
        }
    }
    unsafe { game_write(UNITS_RVA, saved) };
}
