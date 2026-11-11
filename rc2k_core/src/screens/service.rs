//! The service area's readouts: `screens::draw_repair_times`
//! 0x4A70D7 (sixteen repair times as clocks) and `screens::draw_service_time` 0x4A5DE7 (the
//! time available and left, or the overrun and its penalty); also the layout of
//! `screens::draw_service_area` 0x4A5FA2 (its title, labels, damage bars and sprites).

use crate::screens::screen::Screen;
use crate::strings::format::clock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    /// `stru_51E3E4`: the repair times.
    Times,
    /// `stru_51E584`, `stru_51E5A4`: the time available, the time left.
    Available,
    Remaining,
    /// `stru_51E524`, `stru_51E544`: the overrun, the penalty.
    Overrun,
    Penalty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// The penalty.
    Penalty,
    /// `seconds`, `dword_7559EC`: work variables.
    A,
    B,
}

/// The repair times' columns and rows: left column first in each row.
const COLUMNS: [i32; 2] = [0x6A, 0x1B9];
const ROWS: [i32; 8] = [0x6E, 0x8B, 0xA8, 0xC5, 0xE2, 0xFF, 0x11C, 0x1C4];

/// The sixteen times.
pub fn repair_times<S: Screen<Font = Font>>(s: &mut S, times: &[u32; 16]) -> Result<(), S::Error> {
    for (i, &t) in times.iter().enumerate() {
        let (x, y) = (COLUMNS[i % 2], ROWS[i / 2]);
        s.line(Font::Times, &clock(t), x, y)?;
    }
    Ok(())
}

/// The free time the service area gives (`service::state::FREE_SERVICE_TIME`).
pub const FREE_SERVICE_TIME: i32 = 900;
const LEFT_X: i32 = 0x42;
const RIGHT_X: i32 = 0x2DE;
const Y: i32 = 0x2C;

/// "<label>:  <value as a clock>".
fn readout<S: Screen<Font = Font>>(
    s: &mut S,
    (id, font): (u32, Font),
    value: i32,
    x: i32,
) -> Result<bool, S::Error> {
    let mut line = s.string_bytes(id);
    line.extend_from_slice(b":  ");
    line.extend(clock(value as u32));
    Ok(s.line(font, &line, x, Y)?.too_wide)
}

/// The two readouts from the time budget and the repairs' time; the carry of the last
/// text. An overrun of 0 after its 60 is added (only by a wrap) is the original's fatal
/// error 0x80701010 (its division's guard).
pub fn service_time<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    budget: i32,
    repairs: i32,
) -> Result<bool, S::Error> {
    let available = budget.wrapping_sub(FREE_SERVICE_TIME);
    s.keep(Work::Penalty, 0);
    s.keep(Work::A, available as u32);
    if budget.wrapping_sub(repairs) >= FREE_SERVICE_TIME {
        readout(s, (0x48D, Font::Available), available, LEFT_X)?;
        let remaining = available.wrapping_sub(repairs);
        s.keep(Work::B, remaining as u32);
        return readout(s, (0x48E, Font::Remaining), remaining, RIGHT_X);
    }
    let mut overrun = FREE_SERVICE_TIME.wrapping_sub(budget.wrapping_sub(repairs));
    s.keep(Work::B, overrun as u32);
    readout(s, (0x490, Font::Overrun), overrun, LEFT_X)?;
    if overrun != 0 {
        overrun = overrun.wrapping_add(60);
        s.keep(Work::B, overrun as u32);
    }
    if overrun == 0 {
        return Err(s.fatal(0x8070_1010, ""));
    }
    let penalty = (overrun as u32 / 60).wrapping_mul(10);
    s.keep(Work::Penalty, penalty);
    readout(s, (0x491, Font::Penalty), penalty as i32, RIGHT_X)
}

/// The service area's fonts (from `screens::draw_service_area`
/// 0x4A5FA2, whose layout follows).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AreaFont {
    /// `stru_51E8A4`: the title.
    Title,
    /// `stru_51E6A4`: the headings.
    Heading,
    /// `stru_51E3E4`: the parts' labels (the repair times' font).
    Label,
    /// `stru_51E3C4`: the small labels, the chosen tyre.
    Small,
    /// `stru_51E404`.
    Wide,
    /// `stru_51E6E4`: the total.
    Total,
}

/// A piece of the service area's layout, in the original's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AreaStep {
    /// (string, font, x, y)
    Label(u32, AreaFont, i32, i32),
    /// (string, font, x, y): drawn only if the car's class (player +0x90) is not 0.
    LabelIfClass(u32, AreaFont, i32, i32),
    /// (x, y, the part whose damage is the percentage: `Player::damage`'s index)
    DamageBar(i32, i32, usize),
    /// (glyph of the sprite table, x, y): drawn with the palette the last label left.
    Sprite(u32, i32, i32),
}

/// The title's strings, before and after the car's number; where it is drawn.
pub const AREA_TITLE: (u32, u32) = (0x190, 0x486);
pub const AREA_TITLE_AT: (i32, i32) = (0x190, 0xE);
/// The damage bars: width, fill and end glyphs.
pub const DAMAGE_BAR: (i32, u32, u32) = (0xA0, 0x86, 0x87);
/// The chosen tyre's name: this string plus the tyre, and where.
pub const TYRE_FIRST_STRING: u32 = 0x4B0;
pub const TYRE_AT: (i32, i32) = (0x238, 0x152);

/// The title: "<before> <the car's index + 1> <after>" with its NUL.
pub fn area_title(before: &[u8], index: u32, after: &[u8]) -> Vec<u8> {
    use crate::strings::format::{Pad, decimal};
    let mut title = before.to_vec();
    title.push(b' ');
    title.extend(decimal(index.wrapping_add(1), Pad::None));
    title.push(b' ');
    title.extend_from_slice(after);
    title.push(0);
    title
}

pub const AREA_STEPS: [AreaStep; 83] = [
    AreaStep::Label(0x488, AreaFont::Heading, 400, 68),
    AreaStep::DamageBar(236, 100, 0),
    AreaStep::DamageBar(571, 100, 1),
    AreaStep::DamageBar(236, 129, 2),
    AreaStep::DamageBar(571, 129, 3),
    AreaStep::DamageBar(236, 158, 4),
    AreaStep::DamageBar(571, 158, 5),
    AreaStep::DamageBar(236, 187, 6),
    AreaStep::DamageBar(571, 187, 7),
    AreaStep::DamageBar(236, 216, 8),
    AreaStep::DamageBar(571, 216, 9),
    AreaStep::DamageBar(236, 245, 10),
    AreaStep::DamageBar(571, 245, 11),
    AreaStep::DamageBar(236, 274, 12),
    AreaStep::DamageBar(571, 274, 13),
    AreaStep::Label(0x492, AreaFont::Label, 106, 97),
    AreaStep::Label(0x499, AreaFont::Label, 441, 97),
    AreaStep::Label(0x493, AreaFont::Label, 106, 126),
    AreaStep::Label(0x49A, AreaFont::Label, 441, 126),
    AreaStep::Label(0x494, AreaFont::Label, 106, 155),
    AreaStep::Label(0x49B, AreaFont::Label, 441, 155),
    AreaStep::LabelIfClass(0x578, AreaFont::Small, 319, 159),
    AreaStep::Label(0x495, AreaFont::Label, 106, 184),
    AreaStep::Label(0x49C, AreaFont::Label, 441, 184),
    AreaStep::Label(0x496, AreaFont::Label, 106, 213),
    AreaStep::Label(0x49D, AreaFont::Label, 441, 213),
    AreaStep::Label(0x497, AreaFont::Label, 106, 242),
    AreaStep::Label(0x49E, AreaFont::Label, 441, 242),
    AreaStep::Label(0x498, AreaFont::Label, 106, 271),
    AreaStep::Label(0x49F, AreaFont::Label, 441, 271),
    AreaStep::Label(0x4A0, AreaFont::Label, 70, 338),
    AreaStep::Label(0x4A1, AreaFont::Wide, 397, 338),
    AreaStep::Label(0x487, AreaFont::Heading, 400, 353),
    AreaStep::Label(0x4A2, AreaFont::Label, 106, 382),
    AreaStep::Label(0x4A5, AreaFont::Label, 441, 382),
    AreaStep::Label(0x4A8, AreaFont::Label, 106, 438),
    AreaStep::Label(0x4AB, AreaFont::Label, 441, 438),
    AreaStep::Sprite(0x68, 67, 97),
    AreaStep::Sprite(0x69, 402, 97),
    AreaStep::Sprite(0x6A, 67, 126),
    AreaStep::Sprite(0x6B, 402, 126),
    AreaStep::Sprite(0x6C, 67, 155),
    AreaStep::Sprite(0x6D, 402, 155),
    AreaStep::Sprite(0x6E, 67, 184),
    AreaStep::Sprite(0x6F, 402, 184),
    AreaStep::Sprite(0x70, 67, 213),
    AreaStep::Sprite(0x71, 402, 213),
    AreaStep::Sprite(0x72, 67, 242),
    AreaStep::Sprite(0x73, 402, 242),
    AreaStep::Sprite(0x74, 67, 271),
    AreaStep::Sprite(0x75, 402, 271),
    AreaStep::Sprite(0x7B, 70, 301),
    AreaStep::Sprite(0x7C, 136, 301),
    AreaStep::Sprite(0x7D, 202, 301),
    AreaStep::Sprite(0x7E, 268, 301),
    AreaStep::Sprite(0x7F, 334, 301),
    AreaStep::Sprite(0x80, 405, 301),
    AreaStep::Sprite(0x81, 460, 301),
    AreaStep::Sprite(0x82, 515, 301),
    AreaStep::Sprite(0x83, 570, 301),
    AreaStep::Sprite(0x84, 625, 301),
    AreaStep::Sprite(0x85, 680, 301),
    AreaStep::Sprite(0x6F, 67, 382),
    AreaStep::Sprite(0x73, 402, 382),
    AreaStep::Sprite(0x71, 67, 438),
    AreaStep::Sprite(0x71, 402, 438),
    AreaStep::Sprite(0x7A, 70, 412),
    AreaStep::Sprite(0x7A, 405, 412),
    AreaStep::Sprite(0x7A, 70, 468),
    AreaStep::Sprite(0x7A, 405, 468),
    AreaStep::Label(0x48F, AreaFont::Total, 340, 551),
    AreaStep::Label(0x489, AreaFont::Heading, 233, 494),
    AreaStep::Label(0x48A, AreaFont::Heading, 568, 494),
    AreaStep::Label(0x48B, AreaFont::Heading, 233, 522),
    AreaStep::Label(0x48C, AreaFont::Heading, 568, 522),
    AreaStep::Label(0x4A3, AreaFont::Small, 98, 415),
    AreaStep::Label(0x4A4, AreaFont::Small, 367, 415),
    AreaStep::Label(0x4A9, AreaFont::Small, 98, 471),
    AreaStep::Label(0x4AA, AreaFont::Small, 367, 471),
    AreaStep::Label(0x4A6, AreaFont::Small, 433, 415),
    AreaStep::Label(0x4A7, AreaFont::Small, 702, 415),
    AreaStep::Label(0x4AC, AreaFont::Small, 433, 471),
    AreaStep::Label(0x4AD, AreaFont::Small, 702, 471),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_layout() {
        assert_eq!(area_title(b"Car", 2, b"service"), b"Car 3 service\0");
        assert_eq!(
            AREA_STEPS
                .iter()
                .filter(|s| matches!(s, AreaStep::DamageBar(..)))
                .count(),
            14
        );
    }
}
