//! Appends a speed given in tenths of a mile per hour with one decimal, "Q.R", converted to km/h
//! when the game is set to metric units, optionally followed by the unit label ("km/h" on a metric
//! game). A value above `miles_to_km::MAX_INPUT` in metric mode is refused.

use crate::errors::BadInput;
use crate::strings::units::{figure_bytes, setting};

/// `units` is the game's unit setting (0 = imperial). Refuses what `miles_to_km` refuses, and a
/// setting past 1.
pub fn append_speed(
    o: crate::strings::append_string_by_id::StringOwners<'_>,
    tenths: u32,
    with_label: bool,
    units: u32,
) -> Result<Vec<u8>, BadInput> {
    let mut figure = rc2k_core::math::units::speed(tenths, setting(units)?).ok_or(BadInput(
        "result does not fit 32 bits: the original's div traps",
    ))?;
    if !with_label {
        figure.label = None;
    }
    figure_bytes(o, figure)
}
