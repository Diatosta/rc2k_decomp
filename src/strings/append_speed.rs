//! `strings::append_speed` (0x401B10): appends a speed given in tenths of a mile per hour with
//! one decimal, "Q.R", converted to km/h when the game is set to metric units, optionally
//! followed by the unit label ("km/h" on a metric game).
//!
//! Takes tenths of a mile per hour, non-zero to append the unit label, and the cursor. Returns
//! the new cursor.
//! In metric mode a value above `miles_to_km::MAX_INPUT` makes the original trap: refused.
//!
//! Screen: confirmed on the stage selection screen ("Escolha Etapa"), where `draw_stage_records`
//! uses it for the top speed, shown as "187.3km/h".

use crate::errors::BadInput;
use crate::strings::units::{figure_bytes, setting};

// ---- Rust version ----

/// `units` is the game's unit setting (0 = imperial). Refuses what `miles_to_km` refuses, and
/// a setting past 1. Converted: `rc2k_core::math::units::speed`.
pub fn append_speed(tenths: u32, with_label: bool, units: u32) -> Result<Vec<u8>, BadInput> {
    let mut figure = rc2k_core::math::units::speed(tenths, setting(units)?).ok_or(BadInput(
        "result does not fit 32 bits: the original's div traps",
    ))?;
    if !with_label {
        figure.label = None;
    }
    figure_bytes(figure)
}
