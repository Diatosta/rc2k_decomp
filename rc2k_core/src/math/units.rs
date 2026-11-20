//! Unit conversions (Celsius to Fahrenheit, hhmm to minutes, miles to km) and the figures
//! shown with a unit.

use crate::strings::format::{Pad, decimal};

/// The player's choice of units (setting 0 imperial, 1 metric). Figures are kept in miles,
/// Celsius, bhp, lb, lb.ft and 1/100 yd and converted for the metric display.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Units {
    Imperial,
    Metric,
}

impl Units {
    /// From the game's setting; `None` past 1 (the label tables have two entries).
    pub fn from_setting(setting: u32) -> Option<Self> {
        match setting {
            0 => Some(Units::Imperial),
            1 => Some(Units::Metric),
            _ => None,
        }
    }

    fn pick(self, imperial: u16, metric: u16) -> u16 {
        match self {
            Units::Imperial => imperial,
            Units::Metric => metric,
        }
    }

    /// The temperature unit's label: "F" / "C".
    pub(crate) fn temperature_label(self) -> u16 {
        self.pick(0x576, 0x575)
    }
    /// The speed unit's label: "mph" / "km/h".
    pub(crate) fn speed_label(self) -> u16 {
        self.pick(0x573, 0x574)
    }
    /// The power unit's label: "bhp" / "kW".
    pub(crate) fn power_label(self) -> u16 {
        self.pick(0x557, 0x558)
    }
    /// The torque unit's label.
    pub(crate) fn torque_label(self) -> u16 {
        self.pick(0x55C, 0x55D)
    }
    /// The weight unit's label.
    pub(crate) fn weight_label(self) -> u16 {
        self.pick(0x55F, 0x55E)
    }
}

/// The string shown for a car figure that is not known ("n/a").
pub(crate) const NOT_KNOWN_LABEL: u16 = 0x578;
/// The string after a length in millimetres.
pub(crate) const MM_LABEL: u16 = 0x55A;

/// A figure as shown: its text, then the text of string `label` if there is one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Figure {
    pub text: Vec<u8>,
    pub label: Option<u16>,
}

/// `value` times `factor` / 10000, rounded to the nearest (the car figures' metric
/// conversions; `factor` below 10000, so it fits).
fn scaled(value: u32, factor: u32) -> u32 {
    ((value as u64 * factor as u64 + 5_000) / 10_000) as u32
}

/// "whole.tenth" of a value in tenths.
fn tenths_text(value: u32) -> Vec<u8> {
    let mut out = decimal(value / 10, Pad::None);
    out.push(b'.');
    out.extend(decimal(value % 10, Pad::None));
    out
}

/// A speed given in tenths of a mile per hour, with one decimal ("12.3" and its label);
/// `None` when the km/h do not fit 32 bits.
pub fn speed(tenths: u32, units: Units) -> Option<Figure> {
    let value = match units {
        Units::Metric => miles_to_km(tenths)?,
        Units::Imperial => tenths,
    };
    Some(Figure {
        text: tenths_text(value),
        label: Some(units.speed_label()),
    })
}

/// A distance given in hundredths of a mile, with two decimals and no label; `None` when
/// the km do not fit 32 bits.
pub(crate) fn distance_2dp(hundredths: u32, units: Units) -> Option<Vec<u8>> {
    let value = match units {
        Units::Metric => miles_to_km(hundredths)?,
        Units::Imperial => hundredths,
    };
    let mut out = decimal(value / 100, Pad::None);
    out.push(b'.');
    out.extend(decimal(value % 100, Pad::Zeros(2)));
    Some(out)
}

/// A temperature given in whole degrees Celsius, in the player's units with its label.
pub fn temperature(celsius: i32, units: Units) -> Figure {
    let degrees = match units {
        Units::Imperial => celsius_to_fahrenheit(celsius),
        Units::Metric => celsius,
    };
    let mut text = Vec::new();
    if degrees < 0 {
        text.push(b'-');
    }
    text.extend(decimal(degrees.unsigned_abs(), Pad::None));
    Figure {
        text,
        label: Some(units.temperature_label()),
    }
}

/// A car's power in tenths of a bhp: whole bhp or kW.
pub fn power(tenths: u32, units: Units) -> Figure {
    let value = match units {
        Units::Metric => scaled(tenths, 7460),
        Units::Imperial => tenths,
    };
    Figure {
        text: decimal(value / 10, Pad::None),
        label: Some(units.power_label()),
    }
}

/// A car's length in hundredths of a yard: inches with two decimals and `"`, or mm with
/// one decimal; "n/a" for a figure whose second byte is 0xFF.
pub fn length(value: u32, units: Units) -> Figure {
    if (value >> 8) & 0xFF == 0xFF {
        return Figure {
            text: Vec::new(),
            label: Some(NOT_KNOWN_LABEL),
        };
    }
    match units {
        Units::Metric => Figure {
            text: tenths_text(scaled(value, 9144)),
            label: Some(MM_LABEL),
        },
        Units::Imperial => {
            let hundredths = (value as u64 * 36 / 100) as u32;
            let mut text = decimal(hundredths / 100, Pad::None);
            text.push(b'.');
            text.extend(decimal(hundredths % 100, Pad::Zeros(2)));
            text.push(b'"');
            Figure { text, label: None }
        }
    }
}

/// A car's weight in tenths of a lb: whole lb or kg.
pub fn weight(tenths: u32, units: Units) -> Figure {
    let value = match units {
        Units::Metric => scaled(tenths, 4536),
        Units::Imperial => tenths,
    };
    Figure {
        text: decimal(value / 10, Pad::None),
        label: Some(units.weight_label()),
    }
}

/// A car's torque in tenths of a lb.ft: whole lb.ft, or kg.m with a tenth.
pub fn torque(tenths: u32, units: Units) -> Figure {
    let text = match units {
        Units::Metric => tenths_text(scaled(tenths, 1383)),
        Units::Imperial => decimal(tenths / 10, Pad::None),
    };
    Figure {
        text,
        label: Some(units.torque_label()),
    }
}

/// Whole degrees Celsius in Fahrenheit (rounding toward 0).
pub(crate) fn celsius_to_fahrenheit(celsius: i32) -> i32 {
    ((celsius as i64 * 9 / 5) as i32).wrapping_add(32)
}

/// A time of day written as hours * 100 + minutes (1300 for 13:00) in minutes: the last two
/// decimal digits count once, everything above them 60 times.
pub(crate) fn hhmm_to_minutes(hhmm: u32) -> u32 {
    (hhmm / 100) * 60 + hhmm % 100
}

/// The largest distance `miles_to_km` takes: above it the result does not fit 32 bits.
pub(crate) const MAX_MILES: u32 = (((1u64 << 32) * 10_000 - 1 - 5_000) / 16_093) as u32;

/// A distance in miles (any fixed-point scale) in kilometres, rounded to the nearest:
/// times 1.6093. `None` past `MAX_MILES`.
pub fn miles_to_km(miles: u32) -> Option<u32> {
    (miles <= MAX_MILES).then(|| ((miles as u64 * 16_093 + 5_000) / 10_000) as u32)
}

#[cfg(test)]
mod tests;
