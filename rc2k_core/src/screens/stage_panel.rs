//! The stage panel's body, shared by every page that shows it: the start, finish and total
//! times, the length, the road surface, the weather, the description and the numbered line.

use crate::Invalid;
use crate::math::units::{Units, distance_2dp, hhmm_to_minutes, speed, temperature};
use crate::screens::best_scores::{car_glyph, flag_glyph};
use crate::screens::screen::{Buffer, Screen};
use crate::strings::format::{Pad, clock, decimal, fraction, percent, time};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    Label,
    Value,
    /// The surface and weather headings.
    Heading,
    /// The rally stage page's headings.
    RallyHeading,
    Description,
    Number,
    /// The stage heading's.
    Title,
    /// The record rows' time (`FontObjects.objects[14].first_char`, the rally headings' record).
    Time,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// The entry's stage number (two of the panels keep it).
    Entry34,
}

/// What the panel shows of the stage's track-pack entry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    /// Minutes (from the durations table, by the entry's +8).
    pub duration: u32,
    /// Hundredths of a mile; 0 none.
    pub length: u32,
    /// The first of three description strings.
    pub description: u32,
    /// The weather's string.
    pub weather: u32,
    /// Degrees Celsius.
    pub temperature: i32,
    /// Tenths of a mile per hour.
    pub wind: u32,
    /// The surface percentages, entry +0x24, +0x28, +0x2C, +0x30.
    pub surfaces: [u32; 4],
    /// Entry +0x34: the stage's number (what `race::single_rally` looks the entry up by); kept in
    /// `Work::Entry34` by two of the panels.
    pub stage_number: u32,
    pub number: u32,
    /// The number's label is string 0x358 (else 0x359).
    pub number_label_one: bool,
}

// Columns: a label ends at LABEL_x, its value starts at VALUE_x.
const LEFT_LABEL_X: i32 = 0x211;
const LEFT_VALUE_X: i32 = 0x215;
const RIGHT_LABEL_X: i32 = 0x2B8;
const RIGHT_VALUE_X: i32 = 0x2BC;
const CENTRE_X: i32 = 0x266;

/// The strings the panel draws.
pub const LABELS: [u32; 16] = [
    0x34D, 0x34E, 0x356, 0x34F, 0x56F, 0x570, 0x345, 0x346, 0x347, 0x348, 0x349, 0x354, 0x355,
    0x352, 0x358, 0x359,
];

/// A time of day (hours * 100 + minutes) as "H:MM".
fn time_of_day(hhmm: u32) -> Vec<u8> {
    clock(hhmm_to_minutes(hhmm))
}

/// The body for a stage starting at `start` (hours * 100 + minutes): `entry` reads the stage's
/// entry when the page needs it. `units` is the game's setting (0 imperial).
pub fn body<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    start: u32,
    entry: &dyn Fn() -> Result<Entry, S::Error>,
    units: u32,
    heading: Font,
    keep_entry_34: bool,
) -> Result<(), S::Error> {
    let setting = || Units::from_setting(units).ok_or(Invalid("a unit setting with no label"));

    s.string(Font::Label, 0x34D, LEFT_LABEL_X, 0xD1)?;
    s.line(Font::Value, &time_of_day(start), LEFT_VALUE_X, 0xD1)?;

    let e = entry()?;
    let finish = e.duration.wrapping_add(start);
    s.line(Font::Value, &time_of_day(finish), RIGHT_VALUE_X, 0xD1)?;
    s.string(Font::Label, 0x34E, RIGHT_LABEL_X, 0xD1)?;

    s.line(Font::Value, &time_of_day(e.duration), LEFT_VALUE_X, 0xE8)?;
    s.string(Font::Label, 0x356, LEFT_LABEL_X, 0xE8)?;
    if keep_entry_34 {
        s.keep(Work::Entry34, e.stage_number);
    }

    // Length and its unit; a length of 0 leaves the line empty (two NULs are written).
    if e.length == 0 {
        s.store(Buffer::Line, &[0, 0])?;
        s.draw(Font::Value, Buffer::Line, RIGHT_VALUE_X, 0xE8)?;
    } else {
        let metric = if units != 0 {
            Units::Metric
        } else {
            Units::Imperial
        };
        let mut line = distance_2dp(e.length, metric).ok_or(Invalid(
            "result does not fit 32 bits: the original's div traps",
        ))?;
        line.extend(s.string_bytes(if units != 0 { 0x570 } else { 0x56F }));
        s.line(Font::Value, &line, RIGHT_VALUE_X, 0xE8)?;
    }
    s.string(Font::Label, 0x34F, RIGHT_LABEL_X, 0xE8)?;

    // The road surface: four percentages, then their labels.
    s.string(heading, 0x345, CENTRE_X, 0x107)?;
    let [a, b, c, d] = e.surfaces;
    for (value, x, y) in [
        (a, LEFT_VALUE_X, 0x120),
        (c, RIGHT_VALUE_X, 0x120),
        (b, LEFT_VALUE_X, 0x137),
        (d, RIGHT_VALUE_X, 0x137),
    ] {
        s.line(Font::Value, &percent(value), x, y)?;
    }
    for (id, x, y) in [
        (0x346, LEFT_LABEL_X, 0x120),
        (0x347, RIGHT_LABEL_X, 0x120),
        (0x348, LEFT_LABEL_X, 0x137),
        (0x349, RIGHT_LABEL_X, 0x137),
    ] {
        s.string(Font::Label, id, x, y)?;
    }

    // The weather: temperature and wind, then the weather's line.
    s.string(heading, 0x354, CENTRE_X, 0x156)?;
    let t = temperature(e.temperature, setting()?);
    let mut line = t.text;
    if let Some(id) = t.label {
        line.extend(s.string_bytes(id as u32));
    }
    s.line(Font::Value, &line, LEFT_VALUE_X, 0x16F)?;
    s.string(Font::Label, 0x355, LEFT_LABEL_X, 0x16F)?;
    let w = speed(e.wind, setting()?).ok_or(Invalid(
        "result does not fit 32 bits: the original's div traps",
    ))?;
    let mut line = w.text;
    if let Some(id) = w.label {
        line.extend(s.string_bytes(id as u32));
    }
    s.line(Font::Value, &line, RIGHT_VALUE_X, 0x16F)?;
    s.string(Font::Label, 0x352, RIGHT_LABEL_X, 0x16F)?;
    s.string(Font::Value, e.weather, 0x198, 0x186)?;

    // The description: three strings in the text buffer.
    let description = s.strings(e.description, 3)?;
    s.store(Buffer::Text, &description)?;
    s.draw(Font::Description, Buffer::Text, 0x198, 0x1BD)?;

    // The numbered line at the bottom left.
    let mut line = s.string_bytes(if e.number_label_one { 0x358 } else { 0x359 });
    line.push(b' ');
    line.extend(decimal(e.number, Pad::None));
    s.line(Font::Number, &line, 0xC0, 0x206)?;
    Ok(())
}

/// A stage line: string `label`, " ", the stage's number of 6, ": ", the stage's name (string
/// `name`, looked up in the track pack after the rest), without a NUL.
pub fn stage_line<E>(
    strings: &mut dyn FnMut(u32) -> Vec<u8>,
    label: u32,
    number: u32,
    name: &dyn Fn() -> Result<u32, E>,
) -> Result<Vec<u8>, E> {
    let mut line = strings(label);
    line.push(b' ');
    line.extend(fraction(number, 6));
    line.extend(b": ");
    let name = name()?;
    line.extend(strings(name));
    Ok(line)
}

/// The stage heading: the stage line with string 0x481 at (0x2EB, 0x91). Returns its text's
/// `too_wide`.
pub fn heading<S: Screen<Font = Font>>(
    s: &mut S,
    number: u32,
    name: &dyn Fn() -> Result<u32, S::Error>,
) -> Result<bool, S::Error> {
    let line = stage_line(&mut |id| s.string_bytes(id), 0x481, number, name)?;
    Ok(s.line(Font::Title, &line, 0x2EB, 0x91)?.too_wide)
}

/// A record row's holder: the nationality, the name, the car and the figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Holder<T> {
    pub nation: u32,
    pub name: T,
    pub car: u32,
    pub figure: u32,
}

const ROW_1_Y: i32 = 0x1FC;
const ROW_2_Y: i32 = 0x213;

/// The two record rows: "<0x34B>:  <0>" and "<0x34C>:  <0>" (x 0x188), then the best time
/// (y 0x1FC) and the top speed (y 0x213, in tenths of a mile per hour at x 0x2A8). Returns the
/// last text's `too_wide`.
pub fn records<S, T>(
    s: &mut S,
    best: Holder<T>,
    top: Holder<T>,
    show_hours: bool,
    units: u32,
) -> Result<bool, S::Error>
where
    S: Screen<Font = Font, Text = T>,
    T: Copy,
{
    for (id, y) in [(0x34B, ROW_1_Y), (0x34C, ROW_2_Y)] {
        let mut line = s.string_bytes(id);
        line.extend(b":  ");
        line.extend(s.string_bytes(0));
        s.line(Font::Label, &line, 0x188, y)?;
        if id == 0x34C {
            s.sprite(flag_glyph(best.nation), 0x18A, ROW_1_Y)?;
        }
    }
    s.text(Font::Value, best.name, 0x1A6, ROW_1_Y)?;
    s.sprite(car_glyph(best.car), 0x266, ROW_1_Y)?;
    s.time_line(Font::Time, &time(best.figure, show_hours), 0x314, ROW_1_Y)?;

    s.sprite(flag_glyph(top.nation), 0x18A, ROW_2_Y)?;
    s.text(Font::Value, top.name, 0x1A6, ROW_2_Y)?;
    s.sprite(car_glyph(top.car), 0x266, ROW_2_Y)?;
    let setting = Units::from_setting(units).ok_or(Invalid("a unit setting with no label"))?;
    let figure = speed(top.figure, setting).ok_or(Invalid(
        "result does not fit 32 bits: the original's div traps",
    ))?;
    let mut line = figure.text;
    if let Some(id) = figure.label {
        line.extend(s.string_bytes(id as u32));
    }
    Ok(s.line(Font::Value, &line, 0x2A8, ROW_2_Y)?.too_wide)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::screen::Drawn;

    #[derive(Default)]
    struct Record(Vec<String>, Vec<u8>);

    impl Screen for Record {
        type Font = Font;
        type Text = u32;
        type Work = Work;
        type Error = Invalid;
        fn text(&mut self, f: Font, t: u32, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("{f:?} t{t} {x:#x} {y:#x}"));
            Ok(Drawn { ..Drawn::default() })
        }
        fn store(&mut self, _: Buffer, b: &[u8]) -> Result<(), Invalid> {
            self.1 = b.to_vec();
            Ok(())
        }
        fn draw(&mut self, f: Font, _: Buffer, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!(
                "{f:?} {:?} {x:#x} {y:#x}",
                String::from_utf8_lossy(&self.1)
            ));
            Ok(Drawn::default())
        }
        fn draw_clock(&mut self, f: Font, _: Buffer, x: i32, y: i32) -> Result<(), Invalid> {
            self.0.push(format!("clock {f:?} {x:#x} {y:#x}"));
            Ok(())
        }
        fn draw_time(&mut self, f: Font, _: Buffer, x: i32, y: i32) -> Result<(), Invalid> {
            self.0.push(format!(
                "time {f:?} {:?} {x:#x} {y:#x}",
                String::from_utf8_lossy(&self.1)
            ));
            Ok(())
        }
        fn string(&mut self, f: Font, id: u32, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("{f:?} s{id:x} {x:#x} {y:#x}"));
            Ok(Drawn::default())
        }
        fn string_bytes(&mut self, id: u32) -> Vec<u8> {
            format!("<{id:x}>").into_bytes()
        }
        fn strings(&mut self, first: u32, n: u32) -> Result<Vec<u8>, Invalid> {
            Ok(format!("d{first:x}x{n}\0").into_bytes())
        }
        fn sprite(&mut self, g: u32, x: i32, y: i32) -> Result<(), Invalid> {
            self.0.push(format!("sprite {g:#x} {x:#x} {y:#x}"));
            Ok(())
        }
        fn bar(&mut self, b: crate::screens::screen::Bar) -> Result<(), Invalid> {
            self.0.push(format!("bar {b:?}"));
            Ok(())
        }
        fn fatal(&mut self, _: u32, message: &'static str) -> Invalid {
            Invalid(message)
        }
        fn keep(&mut self, w: Work, v: u32) {
            self.0.push(format!("keep {w:?} {v}"));
        }
    }

    #[test]
    fn panel() {
        let entry = Entry {
            duration: 90,
            length: 1234,
            surfaces: [1, 2, 3, 4],
            number: 7,
            ..Entry::default()
        };
        let mut s = Record::default();
        body(&mut s, 1330, &|| Ok(entry), 0, Font::Heading, true).unwrap();
        assert_eq!(s.0[1], "Value \"13:30\\0\" 0x215 0xd1");
        assert!(s.0.contains(&"Value \"12.34<56f>\\0\" 0x2bc 0xe8".to_string()));
        assert!(s.0.contains(&"Value \"3%\\0\" 0x2bc 0x120".to_string()));
        assert_eq!(s.0.last().unwrap(), "Number \"<359> 7\\0\" 0xc0 0x206");
        let mut s = Record::default();
        let none = Entry { length: 0, ..entry };
        body(&mut s, 0, &|| Ok(none), 0, Font::Heading, false).unwrap();
        assert!(s.0.contains(&"Value \"\\0\\0\" 0x2bc 0xe8".to_string()));
        let mut s = Record::default();
        assert!(body(&mut s, 0, &|| Ok(entry), 2, Font::Heading, false).is_err());
    }

    #[test]
    fn records_and_heading() {
        let mut s = Record::default();
        heading(&mut s, 3, &|| Ok(0x99)).unwrap();
        assert_eq!(s.0, ["Title \"<481> 3/6: <99>\\0\" 0x2eb 0x91"]);
        let best = Holder {
            nation: 1,
            name: 10,
            car: 0x19,
            figure: 6000,
        };
        let top = Holder {
            nation: 2,
            name: 20,
            car: 3,
            figure: 1234,
        };
        let mut s = Record::default();
        records(&mut s, best, top, false, 0).unwrap();
        assert_eq!(s.0[0], "Label \"<34b>:  <0>\\0\" 0x188 0x1fc");
        assert_eq!(s.0[2], "sprite 0x1d 0x18a 0x1fc");
        assert_eq!(s.0[4], "sprite 0x5c 0x266 0x1fc");
        assert_eq!(s.0[6], "sprite 0x1e 0x18a 0x213");
        assert_eq!(s.0[8], "sprite 0x47 0x266 0x213");
        assert_eq!(s.0[9], "Value \"123.4<573>\\0\" 0x2a8 0x213");
    }
}
