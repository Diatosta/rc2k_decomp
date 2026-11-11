//! The controller page (from `screens::draw_controller_page`
//! 0x46A454): the player's device, its mode and strength, the sensitivity, two sliders and
//! the stick's place now; and `screens::draw_background_band` 0x4698C4, the background's
//! band drawn again.

use crate::screens::notes::{NO_CONTROLLER, Note};
use crate::screens::screen::{Edge, Screen};
use crate::strings::format::percent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    /// `stru_51E8C4`.
    Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// `seconds`: the string drawn.
    Seconds,
    /// `dword_755A0C`: the device's record (as the game keeps it).
    Record,
    /// `dword_7559DC` .. `dword_7559E8`: the sliders' ends.
    FirstLeft,
    FirstRight,
    SecondLeft,
    SecondRight,
    /// `dword_7559EC`: the stick's place; `dword_7559F0`: its buttons.
    Place,
    Buttons,
}

/// The player's controller as the page shows it.
#[derive(Clone, Copy, Debug)]
pub struct Controller<T> {
    /// The device record's address and its name (+0x30).
    pub record: u32,
    pub name: T,
    /// The record's +0x1C is 1: a strength.
    pub has_strength: bool,
    /// The player's +0x3C, +0x48, +0x34, +0x40, +0x44.
    pub mode: u32,
    pub strength: u32,
    pub sensitivity: u32,
    pub first: u32,
    pub second: u32,
    /// The first device's x and buttons now.
    pub stick_x: u32,
    pub buttons: u32,
}

fn note<S: Screen<Font = Font, Work = Work>>(s: &mut S, n: &Note) -> Result<(), S::Error> {
    s.string(Font::Value, n.id, n.x, n.y).map(|_| ())
}

/// The page: with no controller items 3 .. 8 greyed out and "none"; else them choosable,
/// the device's name, the mode (string 0x56D + mode), item 4 only for a device with a
/// strength and the strength as a percent (or "none"), the sensitivity (string 0x3A1 + it)
/// and sprite 5 after it; the sliders' fills clipped in (the clip's left and right put
/// back) and their ends; the stick's place (x x 100 / 256 from 0x21A; sprite 9 with a
/// button down, else 8). An x whose x 100 is 0 is the original's fatal error 0x80701010.
pub fn controller_page<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    c: Option<&Controller<S::Text>>,
) -> Result<(), S::Error> {
    let Some(c) = c else {
        for n in 3..=8 {
            s.item(n, false)?;
        }
        return note(s, &NO_CONTROLLER);
    };
    for n in 3..=8 {
        s.item(n, true)?;
    }
    s.keep(Work::Record, c.record);
    s.text(Font::Value, c.name, 0x1BC, 0xD5)?;
    let mode = c.mode.wrapping_add(0x56D);
    s.keep(Work::Seconds, mode);
    s.string(Font::Value, mode, 0x1BC, 0xF0)?;
    s.item(4, c.has_strength)?;
    if !c.has_strength || (c.strength as i32) <= 0 {
        s.string(Font::Value, 0x56D, 0x1BC, 0x10B)?;
    } else {
        s.line(Font::Value, &percent(c.strength), 0x1BC, 0x10B)?;
    }
    let sensitivity = c.sensitivity.wrapping_add(0x3A1);
    s.keep(Work::Seconds, sensitivity);
    s.string(Font::Value, sensitivity, 0x1BC, 0x126)?;
    s.sprite(5, 0x1B8, 0x14C)?;
    let (right, left) = (s.clip_edge(Edge::Right)?, s.clip_edge(Edge::Left)?);
    let a = c.first;
    let first_left = 0x1C0u32.wrapping_add(a).wrapping_sub(4);
    s.set_clip_edge(Edge::Right, 0x281u32.wrapping_sub(a).wrapping_sub(1))?;
    s.keep(Work::FirstRight, 0x281u32.wrapping_sub(a));
    s.set_clip_edge(Edge::Left, 0x1C0u32.wrapping_add(a))?;
    s.keep(Work::FirstLeft, first_left);
    let t = 100u32.wrapping_sub(c.second);
    let second_left = 0x1BCu32.wrapping_add(t).wrapping_sub(4);
    let done = (|| {
        s.sprite(3, 0x1BC, 0x14C)?;
        s.set_clip_edge(Edge::Right, 0x285u32.wrapping_sub(t).wrapping_sub(1))?;
        s.keep(Work::SecondRight, 0x285u32.wrapping_sub(t));
        s.set_clip_edge(Edge::Left, 0x1BCu32.wrapping_add(t))?;
        s.keep(Work::SecondLeft, second_left);
        s.sprite(4, 0x1BC, 0x14C)
    })();
    s.set_clip_edge(Edge::Left, left)?;
    s.set_clip_edge(Edge::Right, right)?;
    done?;
    for (glyph, x) in [
        (7u32, first_left),
        (7, 0x281u32.wrapping_sub(a)),
        (6, second_left),
        (6, 0x285u32.wrapping_sub(t)),
    ] {
        s.sprite(glyph, x as i32, 0x14C)?;
    }
    s.keep(Work::Buttons, c.buttons);
    let x = c.stick_x;
    let place = if x == 0 {
        0x21A
    } else {
        let v = if (x as i32) > 0 { x } else { x.wrapping_neg() }.wrapping_mul(100);
        if v == 0 {
            return Err(s.fatal(0x8070_1010, ""));
        }
        if (x as i32) > 0 {
            (v / 0x100).wrapping_add(0x21A)
        } else {
            (v / 0x100)
                .wrapping_neg()
                .wrapping_add(1)
                .wrapping_add(0x21A)
        }
    };
    s.keep(Work::Place, place);
    let glyph = if c.buttons != 0 { 9 } else { 8 };
    s.sprite(glyph, place as i32, 0x156)
}

/// The band (only while `on`, `dword_7554E4` 1): sprite 0xA0 (the background picture) at
/// (0, 0) with the clip's top and bottom at 0xCB and 0x1C6 for that one blit, then put back.
pub fn background_band<S: Screen>(s: &mut S, on: bool) -> Result<(), S::Error> {
    if !on {
        return Ok(());
    }
    let (top, bottom) = (s.clip_edge(Edge::Top)?, s.clip_edge(Edge::Bottom)?);
    s.set_clip_edge(Edge::Top, 0xCB)?;
    s.set_clip_edge(Edge::Bottom, 0x1C6)?;
    let drawn = s.sprite(0xA0, 0, 0);
    s.set_clip_edge(Edge::Bottom, bottom)?;
    s.set_clip_edge(Edge::Top, top)?;
    drawn
}
