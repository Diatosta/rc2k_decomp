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

use crate::errors::{BadInput, Refused};
use crate::framework::{EBX, RawHook, Regs, SelfTest, game_write, rva};
use crate::strings::append_string_by_id::push_string;
use crate::strings::format_u32::format_u32;
use crate::strings::units::units;
use crate::strings::utils::bytes_str;

pub const UNITS_RVA: usize = rva(0x51DE2C); // g_isImperialUnits
/// `word_51EB8C`, `word_51EB90`, `word_51EB94`: a label for each setting.
pub const POWER_LABELS: [u32; 2] = [0x557, 0x558];
pub const TORQUE_LABELS: [u32; 2] = [0x55C, 0x55D];
pub const WEIGHT_LABELS: [u32; 2] = [0x55F, 0x55E];
const NOT_KNOWN: u32 = 0x578;
const MM: u32 = 0x55A;

/// (v * factor + 5000) / 10000, as the original's `mul` and `div`.
pub fn convert(value: u32, factor: u32) -> u32 {
    ((value as u64 * factor as u64 + 5000) / 10000) as u32
}

/// The setting, refused when it is neither of the two labels' places.
fn setting() -> Result<u32, Refused> {
    let units = units();
    if units > 1 {
        return Err(BadInput("a unit setting with no label").into());
    }
    Ok(units)
}

fn label(out: &mut Vec<u8>, id: u32) {
    unsafe { push_string(out, id) };
}

/// `strings::append_power` (0x401CBA).
pub fn append_power(value: u32) -> Result<Vec<u8>, Refused> {
    let units = setting()?;
    let value = if units != 0 {
        convert(value, 7460)
    } else {
        value
    };
    let mut out = format_u32(value / 10, 0);
    label(&mut out, POWER_LABELS[units as usize]);
    Ok(out)
}

/// `strings::append_length` (0x401C42).
pub fn append_length(value: u32) -> Result<Vec<u8>, Refused> {
    let mut out = Vec::new();
    if (value >> 8) & 0xFF == 0xFF {
        label(&mut out, NOT_KNOWN);
        return Ok(out);
    }
    if setting()? != 0 {
        let value = convert(value, 9144);
        out.extend(format_u32(value / 10, 0));
        out.push(b'.');
        out.extend(format_u32(value % 10, 0));
        label(&mut out, MM);
    } else {
        let hundredths = (value as u64 * 36 / 100) as u32;
        out.extend(format_u32(hundredths / 100, 0));
        out.push(b'.');
        out.extend(format_u32(hundredths % 100, 0x12));
        out.push(b'"');
    }
    Ok(out)
}

/// `strings::append_weight` (0x401CF1).
pub fn append_weight(value: u32) -> Result<Vec<u8>, Refused> {
    let units = setting()?;
    let value = if units != 0 {
        convert(value, 4536)
    } else {
        value
    };
    let mut out = format_u32(value / 10, 0);
    label(&mut out, WEIGHT_LABELS[units as usize]);
    Ok(out)
}

/// `strings::append_torque` (0x401D28).
pub fn append_torque(value: u32) -> Result<Vec<u8>, Refused> {
    let units = setting()?;
    let value = if units != 0 {
        convert(value, 1383)
    } else {
        value
    };
    let mut out = format_u32(value / 10, 0);
    if units != 0 {
        out.push(b'.');
        out.extend(format_u32(value % 10, 0));
    }
    label(&mut out, TORQUE_LABELS[units as usize]);
    Ok(out)
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
