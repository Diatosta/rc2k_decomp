//! The game's unit system and the string ids of its unit labels.

use crate::errors::BadInput;
use crate::framework::{game_read, rva};
pub use rc2k_core::math::units::{Figure, Units};

// 0 = imperial (Fahrenheit, miles) and non-zero = metric.
const UNITS_RVA: usize = rva(0x51DE2C);

/// 0 = imperial, non-zero = metric. The game stores Celsius and miles.
pub fn units() -> u32 {
    unsafe { game_read(UNITS_RVA) }
}

/// The setting as the core's `Units`; refused past 1 (the label tables have two entries).
pub fn setting(units: u32) -> Result<Units, BadInput> {
    Units::from_setting(units).ok_or(BadInput("a unit setting with no label"))
}

/// A core `Figure` as the bytes the game appends: the text, then the label's string.
pub fn figure_bytes(figure: Figure) -> Vec<u8> {
    let mut out = figure.text;
    if let Some(id) = figure.label {
        unsafe { crate::strings::append_string_by_id::push_string(&mut out, id as u32) };
    }
    out
}

pub fn is_metric() -> bool {
    units() != 0
}
