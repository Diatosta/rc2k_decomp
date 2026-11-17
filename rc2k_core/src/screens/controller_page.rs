//! The controller page: the player's device, its shifter switch, strength and button pictures; and the background
//! band drawn again.

use crate::screens::notes::{NO_CONTROLLER, Note};
use crate::screens::screen::{Edge, Screen};
use crate::strings::format::percent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {}

/// The player's controller as the page shows it.
#[derive(Clone, Copy, Debug)]
pub struct Controller<T> {
    /// The device's name.
    pub name: T,
    /// The device has a force feedback strength.
    pub has_strength: bool,
    /// The player's shifter switch and force feedback strength.
    pub mode: u32,
    pub strength: u32,
    /// The text id of the style the button pictures are set to.
    pub icons: u32,
}

fn note<S: Screen<Font = Font, Work = Work>>(s: &mut S, n: &Note) -> Result<(), S::Error> {
    s.string(Font::Value, n.id, n.x, n.y).map(|_| ())
}

/// The page: with no controller items 3 .. 7 are greyed out and "none" shown; else the device's
/// name, the shifter switch, the strength (item 4 only for a device with one) and the style of
/// the button pictures.
pub fn controller_page<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    c: Option<&Controller<S::Text>>,
) -> Result<(), S::Error> {
    let Some(c) = c else {
        for n in 3..=7 {
            s.item(n, false)?;
        }
        return note(s, &NO_CONTROLLER);
    };
    for n in 3..=7 {
        s.item(n, true)?;
    }
    s.text(Font::Value, c.name, 0x1BC, 0xD5)?;
    let mode = c.mode.wrapping_add(0x56D);
    s.string(Font::Value, mode, 0x1BC, 0xF0)?;
    s.item(4, c.has_strength)?;
    if !c.has_strength || (c.strength as i32) <= 0 {
        s.string(Font::Value, 0x56D, 0x1BC, 0x10B)
    } else {
        s.line(Font::Value, &percent(c.strength), 0x1BC, 0x10B)
    }?;
    s.string(Font::Value, c.icons, 0x1BC, 0x15C).map(|_| ())
}

/// The band (only while `on`, `ScreenFlow.left_service` 1): sprite 0xA0 at (0, 0) with the
/// clip's top and bottom at 0xCB and 0x1C6 for that blit, then put back.
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
