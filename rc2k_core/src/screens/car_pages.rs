//! The car specification's four figure pages (description, engine, transmission, dimensions):
//! labels and figures, "n/d" for an unknown figure, the Clio's own strings.

use crate::Invalid;
use crate::math::units::{Figure, Units, length, power, torque, weight};
use crate::screens::screen::{Buffer, Screen};
use crate::strings::format::{Pad, decimal};

/// A car's description (0x78 bytes): 34 of them, the last the Clio of the update disc (car
/// 0x21). Figures are imperial, in tenths; strings are string ids.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CarDescription {
    /// The car's number: its pictures, background, slides and stream go by it.
    pub car: u32,
    /// String id: the car's name.
    pub name: u32,
    /// The car's class, 0 .. 3 (a player's `class`, the standings' group).
    pub class: u32,
    /// Cubic centimetres.
    pub capacity: u32,
    /// String id.
    pub engine_type: u32,
    pub power: u32,
    pub power_rpm: u32,
    pub torque: u32,
    pub torque_rpm: u32,
    pub bore: u32,
    pub stroke: u32,
    /// String id.
    pub compression: u32,
    /// String id.
    pub transmission: u32,
    pub drive: u32,
    pub steering: u32,
    pub front_brake_size: u32,
    pub rear_brake_size: u32,
    /// String id.
    pub front_brakes: u32,
    pub rear_brakes: u32,
    pub front_suspension: u32,
    pub rear_suspension: u32,
    pub length: u32,
    pub width: u32,
    pub height: u32,
    pub wheelbase: u32,
    pub front_track: u32,
    pub rear_track: u32,
    pub weight: u32,
    /// String: the first of the seven lines of its description.
    pub description: u32,
    /// Where its figures are (`cars_store::description_table`).
    pub figures: u32,
}

impl CarDescription {
    /// The record from its 120 bytes (little-endian dwords).
    pub const fn from_le_bytes(b: [u8; 120]) -> Self {
        const fn word(b: &[u8; 120], n: usize) -> u32 {
            u32::from_le_bytes([b[4 * n], b[4 * n + 1], b[4 * n + 2], b[4 * n + 3]])
        }
        Self {
            car: word(&b, 0),
            name: word(&b, 1),
            class: word(&b, 2),
            capacity: word(&b, 3),
            engine_type: word(&b, 4),
            power: word(&b, 5),
            power_rpm: word(&b, 6),
            torque: word(&b, 7),
            torque_rpm: word(&b, 8),
            bore: word(&b, 9),
            stroke: word(&b, 10),
            compression: word(&b, 11),
            transmission: word(&b, 12),
            drive: word(&b, 13),
            steering: word(&b, 14),
            front_brake_size: word(&b, 15),
            rear_brake_size: word(&b, 16),
            front_brakes: word(&b, 17),
            rear_brakes: word(&b, 18),
            front_suspension: word(&b, 19),
            rear_suspension: word(&b, 20),
            length: word(&b, 21),
            width: word(&b, 22),
            height: word(&b, 23),
            wheelbase: word(&b, 24),
            front_track: word(&b, 25),
            rear_track: word(&b, 26),
            weight: word(&b, 27),
            description: word(&b, 28),
            figures: word(&b, 29),
        }
    }
}

const _: () = assert!(std::mem::size_of::<CarDescription>() == 0x78);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    Labels,
    Figures,
}

/// No work variables: the description is passed in, not kept in a scratch value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {}

/// The string "n/d".
const NOT_KNOWN: u32 = 0x578;
/// The Clio's car number.
const CLIO: u32 = 0x21;

struct Page<'a, S> {
    s: &'a mut S,
    carry: bool,
    units: u32,
}

impl<S: Screen<Font = Font, Work = Work>> Page<'_, S> {
    fn setting(&self) -> Result<Units, Invalid> {
        Units::from_setting(self.units).ok_or(Invalid("a unit setting with no label"))
    }

    fn bytes(&mut self, figure: Figure) -> Vec<u8> {
        let mut out = figure.text;
        if let Some(id) = figure.label {
            out.extend(self.s.string_bytes(id as u32));
        }
        out
    }

    fn string(&mut self, id: u32, font: Font, x: i32, y: i32) -> Result<(), S::Error> {
        self.carry = self.s.string(font, id, x, y)?.too_wide;
        Ok(())
    }

    fn line(&mut self, line: &[u8], x: i32, y: i32) -> Result<(), S::Error> {
        self.carry = self.s.line(Font::Figures, line, x, y)?.too_wide;
        Ok(())
    }

    /// A figure by `format`, "n/d" for 0.
    fn figure(&mut self, value: u32, kind: Kind) -> Result<Vec<u8>, S::Error> {
        if value == 0 {
            return Ok(self.s.string_bytes(NOT_KNOWN));
        }
        let figure = match kind {
            Kind::Power => power(value, self.setting()?),
            Kind::Torque => torque(value, self.setting()?),
            Kind::Weight => weight(value, self.setting()?),
            // A length whose second byte is 0xFF needs no setting.
            Kind::Length => {
                let units = if (value >> 8) & 0xFF == 0xFF {
                    Units::Imperial
                } else {
                    self.setting()?
                };
                length(value, units)
            }
        };
        Ok(self.bytes(figure))
    }

    /// A figure "@rpm" and string 0x559 after it when the rpm is known.
    fn with_rpm(&mut self, value: u32, rpm: u32, kind: Kind) -> Result<Vec<u8>, S::Error> {
        let mut out = self.figure(value, kind)?;
        if value != 0 && rpm != 0 {
            out.push(b'@');
            out.extend(decimal(rpm, Pad::None));
            out.extend(self.s.string_bytes(0x559));
        }
        Ok(out)
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Power,
    Torque,
    Length,
    Weight,
}

/// Page `page` (0 .. 3) of `d`: returns the last text's `too_wide`. The Clio in notice kind 3
/// editions has a description string replaced first (in `d`). `units` is the game's setting.
pub fn page<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    page: u32,
    d: &mut CarDescription,
    notice_kind: u8,
    units: u32,
) -> Result<bool, S::Error> {
    let clio = notice_kind == 3 && d.car == CLIO;
    let mut p = Page {
        s,
        carry: false,
        units,
    };
    match page {
        0 => {
            if d.car == CLIO {
                if clio {
                    d.description = 0x103F;
                }
                p.string(d.description, Font::Figures, 0x1A4, 0xD5)?;
            } else {
                let text = p.s.strings(d.description, 7)?;
                p.s.store(Buffer::Text, &text)?;
                p.carry = p.s.draw(Font::Figures, Buffer::Text, 0x1A4, 0xD5)?.too_wide;
            }
        }
        1 => {
            let x = 0x244;
            p.string(0x24F, Font::Labels, x, 0x10A)?;
            let line = if d.capacity == 0 {
                p.s.string_bytes(NOT_KNOWN)
            } else {
                let mut line = decimal(d.capacity, Pad::None);
                line.extend(p.s.string_bytes(0x266));
                line
            };
            p.line(&line, x, 0x10A)?;
            p.string(0x250, Font::Labels, x, 0x121)?;
            if clio {
                d.engine_type = 0x103A;
            }
            p.string(d.engine_type, Font::Figures, x, 0x121)?;
            p.string(0x251, Font::Labels, x, 0x14F)?;
            let line = p.with_rpm(d.power, d.power_rpm, Kind::Power)?;
            p.line(&line, x, 0x14F)?;
            p.string(0x252, Font::Labels, x, 0x166)?;
            let line = p.with_rpm(d.torque, d.torque_rpm, Kind::Torque)?;
            p.line(&line, x, 0x166)?;
            p.string(0x253, Font::Labels, x, 0x17D)?;
            let line = p.figure(d.bore, Kind::Length)?;
            p.line(&line, x, 0x17D)?;
            p.string(0x254, Font::Labels, x, 0x194)?;
            let line = p.figure(d.stroke, Kind::Length)?;
            p.line(&line, x, 0x194)?;
            p.string(0x255, Font::Labels, x, 0x1AB)?;
            p.string(d.compression, Font::Figures, x, 0x1AB)?;
        }
        2 => {
            let x = 0x220;
            p.string(0x256, Font::Labels, x, 0xF3)?;
            p.string(d.transmission, Font::Figures, x, 0xF3)?;
            p.string(0x257, Font::Labels, x, 0x10A)?;
            if clio {
                d.drive = 0x103D;
            }
            p.string(d.drive, Font::Figures, x, 0x10A)?;
            p.string(0x258, Font::Labels, x, 0x121)?;
            p.string(d.steering, Font::Figures, x, 0x121)?;
            p.string(0x259, Font::Labels, x, 0x138)?;
            let mut line = p.s.string_bytes(0x271);
            line.extend(p.figure(d.front_brake_size, Kind::Length)?);
            line.push(b' ');
            line.extend(p.s.string_bytes(d.front_brakes));
            line.extend(b"\r\n");
            line.extend(p.s.string_bytes(0x272));
            line.extend(p.figure(d.rear_brake_size, Kind::Length)?);
            line.push(b' ');
            line.extend(p.s.string_bytes(d.rear_brakes));
            line.extend(b"\r\n");
            p.line(&line, x, 0x138)?;
            p.string(0x25A, Font::Labels, x, 0x18C)?;
            let mut line = p.s.string_bytes(0x271);
            line.extend(p.s.string_bytes(d.front_suspension));
            line.extend(b"\r\n");
            line.extend(p.s.string_bytes(0x272));
            line.extend(p.s.string_bytes(d.rear_suspension));
            line.extend(b"\r\n");
            p.line(&line, x, 0x18C)?;
        }
        _ => {
            let x = 0x28B;
            let lengths = [
                d.length,
                d.width,
                d.height,
                d.wheelbase,
                d.front_track,
                d.rear_track,
            ];
            for (n, value) in lengths.into_iter().enumerate() {
                let y = 0x10A + n as i32 * 0x17;
                p.string(0x25B + n as u32, Font::Labels, x, y)?;
                let line = p.figure(value, Kind::Length)?;
                p.line(&line, x, y)?;
            }
            p.string(0x261, Font::Labels, x, 0x194)?;
            let line = p.figure(d.weight, Kind::Weight)?;
            p.line(&line, x, 0x194)?;
        }
    }
    Ok(p.carry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::screen::{Bar, Drawn};

    #[derive(Default)]
    struct Record(Vec<String>, Vec<u8>);

    impl Screen for Record {
        type Font = Font;
        type Text = u32;
        type Work = Work;
        type Error = Invalid;
        fn text(&mut self, _: Font, _: u32, _: i32, _: i32) -> Result<Drawn, Invalid> {
            unreachable!()
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
        fn draw_time(&mut self, _: Font, _: Buffer, _: i32, _: i32) -> Result<(), Invalid> {
            unreachable!()
        }
        fn draw_clock(&mut self, _: Font, _: Buffer, _: i32, _: i32) -> Result<(), Invalid> {
            unreachable!()
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
        fn sprite(&mut self, _: u32, _: i32, _: i32) -> Result<(), Invalid> {
            unreachable!()
        }
        fn keep(&mut self, w: Work, _: u32) {
            match w {}
        }
        fn bar(&mut self, _: Bar) -> Result<(), Invalid> {
            unreachable!()
        }
        fn fatal(&mut self, _: u32, m: &'static str) -> Invalid {
            Invalid(m)
        }
    }

    #[test]
    fn pages() {
        let mut d = CarDescription {
            car: 0x21,
            description: 0x500,
            engine_type: 0x501,
            power: 1500,
            power_rpm: 6000,
            ..CarDescription::default()
        };
        let mut s = Record::default();
        page(&mut s, 0, &mut d, 3, 0).unwrap();
        assert_eq!(s.0, ["Figures s103f 0x1a4 0xd5"]);
        assert_eq!(d.description, 0x103F);
        let mut s = Record::default();
        page(&mut s, 1, &mut d, 3, 0).unwrap();
        assert_eq!(d.engine_type, 0x103A);
        assert_eq!(s.0[1], "Figures \"<578>\\0\" 0x244 0x10a");
        assert!(s.0[5].contains("@6000<559>"), "{}", s.0[5]);
        let mut other = CarDescription {
            car: 3,
            description: 0x600,
            ..CarDescription::default()
        };
        let mut s = Record::default();
        page(&mut s, 0, &mut other, 3, 0).unwrap();
        assert_eq!(s.0, ["Figures \"d600x7\\0\" 0x1a4 0xd5"]);
        assert!(page(&mut Record::default(), 3, &mut other, 0, 2).is_ok());
        let mut s = Record::default();
        other.length = 120;
        assert!(page(&mut s, 3, &mut other, 0, 2).is_err());
    }
}
