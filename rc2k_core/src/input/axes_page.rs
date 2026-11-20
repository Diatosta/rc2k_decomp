//! What the controller's axes pages edit: the axes list's wheel rows, and for one axis its four
//! settings (the dead zone, the sensitivity, the range and the invert of the axis's
//! `AxisSettings`). Left and right change the highlighted row's value.
//!
//! A wheel (`Profile::wheel`) also has a full rotation: the range then shows in degrees of it.

use crate::input::profile::{AXES, AxisSettings, AxisUse, Profile, SENSITIVITY_LIMIT, shaped};
use crate::strings::format::{Pad, decimal};

/// The rows the list has at most.
pub const ROWS: usize = 6;
/// The most the dead zone is set to (per mille).
pub const MAX_DEAD_ZONE: u32 = 900;
/// The least the range is set to (per mille).
pub(crate) const MIN_RANGE: u32 = 100;
/// What one step of the dead zone, the range or a wheel's rotation changes: 1 % and 10 degrees.
pub const STEP: u32 = 10;
/// What one step of the sensitivity changes.
pub(crate) const SENSITIVITY_STEP: i32 = 10;
/// A wheel's rotation, least and most (degrees).
pub(crate) const MIN_ROTATION: u32 = 90;
pub(crate) const MAX_ROTATION: u32 = 1800;

/// The settings of one axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Column {
    DeadZone,
    Sensitivity,
    Range,
    Invert,
}

impl Column {
    /// The settings in the order the axis page lists them.
    pub const ALL: [Column; 4] = [
        Column::DeadZone,
        Column::Sensitivity,
        Column::Range,
        Column::Invert,
    ];
}

/// Column `column` of `settings` one step up or down, to its limits. The invert flips whichever
/// way.
pub fn step(settings: &mut AxisSettings, column: Column, up: bool) {
    match column {
        Column::DeadZone => {
            settings.dead_zone = if up {
                (settings.dead_zone + STEP).min(MAX_DEAD_ZONE)
            } else {
                settings.dead_zone.saturating_sub(STEP)
            };
        }
        Column::Sensitivity => {
            let moved = settings.sensitivity
                + if up {
                    SENSITIVITY_STEP
                } else {
                    -SENSITIVITY_STEP
                };
            settings.sensitivity = moved.clamp(-SENSITIVITY_LIMIT, SENSITIVITY_LIMIT);
        }
        Column::Range => {
            settings.range = if up {
                (settings.range + STEP).min(1000)
            } else {
                settings.range.saturating_sub(STEP).max(MIN_RANGE)
            };
        }
        Column::Invert => settings.invert = !settings.invert,
    }
}

/// A wheel's full rotation one step up or down.
pub fn step_rotation(degrees: u32, up: bool) -> u32 {
    if up {
        (degrees + STEP).min(MAX_ROTATION)
    } else {
        degrees.saturating_sub(STEP).max(MIN_ROTATION)
    }
}

/// The words the values are written with.
#[derive(Clone, Copy, Debug)]
pub struct Words<'a> {
    pub on: &'a [u8],
    pub off: &'a [u8],
    /// A wheel's unit, in the range's "(450 degrees)".
    pub degrees: &'a [u8],
    /// The sensitivity: none (0), above 0 (finer near the rest) and below 0 (quicker).
    pub linear: &'a [u8],
    pub gentle: &'a [u8],
    pub quick: &'a [u8],
}

/// A figure and a word in brackets: "+20 (gentle)".
fn with_word(figure: Vec<u8>, word: &[u8]) -> Vec<u8> {
    let mut out = figure;
    out.extend_from_slice(b" (");
    out.extend_from_slice(word);
    out.push(b')');
    out
}

/// The value of a setting as text (no NUL): a per cent, a signed figure and what it does, a
/// range (for a wheel with the full rotation given, its degrees after the per cent), the invert
/// as `on` or `off`.
pub fn value_text(
    settings: &AxisSettings,
    column: Column,
    rotation: Option<u32>,
    words: &Words,
) -> Vec<u8> {
    let percent = |per_mille: u32| {
        let mut out = decimal(per_mille / 10, Pad::None);
        out.push(b'%');
        out
    };
    match column {
        Column::DeadZone => percent(settings.dead_zone),
        Column::Sensitivity => {
            let figure = decimal(settings.sensitivity.unsigned_abs(), Pad::None);
            match settings.sensitivity {
                0 => words.linear.to_vec(),
                s if s > 0 => with_word([&b"+"[..], &figure].concat(), words.gentle),
                _ => with_word([&b"-"[..], &figure].concat(), words.quick),
            }
        }
        Column::Range => {
            let mut out = percent(settings.range);
            if let Some(rotation) = rotation {
                let mut degrees = decimal(settings.range * rotation / 1000, Pad::None);
                degrees.push(b' ');
                degrees.extend_from_slice(words.degrees);
                out.push(b' ');
                out.push(b'(');
                out.extend(degrees);
                out.push(b')');
            }
            out
        }
        Column::Invert => if settings.invert { words.on } else { words.off }.to_vec(),
    }
}

/// How far an axis is from its rest, for the live bar: 0 ..= 100 per cent (`centred`: of the
/// travel from the middle to the side it is on, `side` -1 left, 1 right, 0 at the middle).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub percent: i32,
    pub side: i32,
    /// Measured from the end a pedal rests at, not from the middle.
    pub pedal: bool,
}

impl Position {
    /// The part of a bar `width` wide that is filled, as its start and length: a pedal's travel
    /// from the left; for a centred axis from the middle to the side it leans.
    pub fn span(self, width: i32) -> (i32, i32) {
        let percent = self.percent.clamp(0, 100);
        if self.pedal {
            return (0, width * percent / 100);
        }
        let half = width / 2;
        let length = half * percent / 100;
        if self.side < 0 {
            (half - length, length)
        } else {
            (half, length)
        }
    }
}

/// Where axis `axis` reads now (`raw`, -32768 ..= 32767) for the bar, as the race takes it: with
/// the axis's invert, dead zone, range and curve applied. An axis the profile has as a pedal is
/// measured from the end it rests at; any other from its middle.
pub fn position(profile: &Profile, axis: u8, raw: i32) -> Position {
    let settings = profile.settings(axis);
    let from_rest_end = [
        profile.bindings.accelerate_axis,
        profile.bindings.brake_axis,
    ]
    .into_iter()
    .flatten()
    .find_map(|using| match using {
        AxisUse::Pedal { axis: a, rest_high } if a == axis => Some(rest_high),
        _ => None,
    });
    let mut unit = f64::from(raw.clamp(-32767, 32767)) / 32767.0;
    if settings.invert {
        unit = -unit;
    }
    let shaped_percent =
        |travel: f64| (shaped(&settings, travel.clamp(0.0, 1.0)) * 100.0).round() as i32;
    match from_rest_end {
        Some(rest_high) => {
            let travel = if rest_high {
                (1.0 - unit) / 2.0
            } else {
                (1.0 + unit) / 2.0
            };
            Position {
                percent: shaped_percent(travel),
                side: 1,
                pedal: true,
            }
        }
        None => Position {
            percent: shaped_percent(unit.abs()),
            side: match unit {
                u if u > 0.01 => 1,
                u if u < -0.01 => -1,
                _ => 0,
            },
            pedal: false,
        },
    }
}

/// How many rows the page shows for a controller with `axes` axes.
pub fn rows(axes: usize) -> usize {
    axes.min(ROWS).min(AXES)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::device_id::DeviceId;
    use crate::input::profile::Kind;

    #[test]
    fn steps_stay_inside_their_limits() {
        let mut s = AxisSettings::default();
        for _ in 0..200 {
            step(&mut s, Column::DeadZone, true);
            step(&mut s, Column::Sensitivity, true);
            step(&mut s, Column::Range, false);
        }
        assert_eq!((s.dead_zone, s.sensitivity, s.range), (900, 100, 100));
        for _ in 0..200 {
            step(&mut s, Column::DeadZone, false);
            step(&mut s, Column::Sensitivity, false);
            step(&mut s, Column::Range, true);
        }
        assert_eq!((s.dead_zone, s.sensitivity, s.range), (0, -100, 1000));
        step(&mut s, Column::Invert, false);
        assert!(s.invert);
        assert_eq!(step_rotation(900, true), 910);
        assert_eq!(step_rotation(MIN_ROTATION, false), MIN_ROTATION);
        assert_eq!(step_rotation(MAX_ROTATION, true), MAX_ROTATION);
    }

    #[test]
    fn the_values_read_as_text() {
        let s = AxisSettings {
            dead_zone: 100,
            range: 800,
            sensitivity: -30,
            invert: true,
        };
        let words = Words {
            on: b"on",
            off: b"off",
            degrees: b"degrees",
            linear: b"linear",
            gentle: b"gentle",
            quick: b"quick",
        };
        let text = |c, wheel| value_text(&s, c, wheel, &words);
        assert_eq!(text(Column::DeadZone, None), b"10%");
        assert_eq!(text(Column::Sensitivity, None), b"-30 (quick)");
        assert_eq!(text(Column::Range, None), b"80%");
        assert_eq!(text(Column::Range, Some(900)), b"80% (720 degrees)");
        assert_eq!(text(Column::Invert, None), b"on");
        let flat = AxisSettings::default();
        assert_eq!(text_of(&flat, &words), b"linear");
        let gentle = AxisSettings {
            sensitivity: 20,
            ..flat
        };
        assert_eq!(text_of(&gentle, &words), b"+20 (gentle)");
    }

    fn text_of(s: &AxisSettings, words: &Words) -> Vec<u8> {
        value_text(s, Column::Sensitivity, None, words)
    }

    #[test]
    fn a_pedal_is_measured_from_its_rest() {
        let id = DeviceId {
            name: "Wheel".into(),
            serial: None,
            nth: 1,
        };
        let mut p = Profile::new(id, Kind::Pad);
        for axis in 0..AXES as u8 {
            *p.settings_mut(axis) = AxisSettings {
                dead_zone: 0,
                ..AxisSettings::default()
            };
        }
        // the pad's triggers rest at the small end
        assert_eq!(position(&p, 5, -32768).percent, 0);
        assert_eq!(position(&p, 5, 32767).percent, 100);
        p.bindings.brake_axis = Some(AxisUse::Pedal {
            axis: 2,
            rest_high: true,
        });
        assert_eq!(position(&p, 2, 32767).percent, 0);
        assert_eq!(position(&p, 2, -32767).percent, 100);
        // a stick axis: from the middle, with its side
        assert_eq!(position(&p, 0, 0).span(100), (50, 0));
        let left = position(&p, 0, -16384);
        assert_eq!(
            (left.percent, left.side, left.span(100)),
            (50, -1, (25, 25))
        );
        assert_eq!(position(&p, 0, 32767).span(100), (50, 50));
        assert_eq!(position(&p, 5, 32767).span(100), (0, 100));
        // the settings shape what the bar shows
        p.settings_mut(0).dead_zone = 500;
        assert_eq!(position(&p, 0, 16384).percent, 0);
        assert_eq!(position(&p, 0, 32767).percent, 100);
        p.settings_mut(0).dead_zone = 0;
        p.settings_mut(0).invert = true;
        assert_eq!(position(&p, 0, 16384).side, -1);
        assert_eq!(rows(8), 6);
        assert_eq!(rows(2), 2);
    }
}
