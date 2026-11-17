//! The controller's axes pages. The list: a live bar for each axis the controller has, then the
//! wheel's switch and full rotation. One axis's page: the axis, its four settings (levels shown
//! on a slider) and the live bar again, so the effect of a setting can be seen.
//!
//! The rows' labels are the menus' items; the values and bars are drawn here, on the controller
//! page's fonts, in the column the other controller pages put their values in.

use crate::input::axes_page::{Column, Position};
use crate::screens::controller_page::{Font, Work};
use crate::screens::screen::{Bar, Screen};

/// The first row's text y and the rows' spacing (the items' own).
pub const FIRST_ROW_Y: i32 = 183;
pub const ROW_SPACING: i32 = 30;
/// The x the values start at, as the other controller pages' do.
pub const VALUE_X: i32 = 0x1BC;
/// The live bar's width on the list and on an axis's page, and its height.
pub const LIST_BAR_WIDTH: i32 = 140;
pub const PAGE_BAR_WIDTH: i32 = 200;
pub const BAR_HEIGHT: i32 = 18;
/// The bar's track and fill colours, and the mark of a centred axis's middle.
pub const TRACK_RGB: u32 = 0x0004_0A2E;
pub const FILL_RGB: u32 = 0x0020_B4EC;
pub const MIDDLE_RGB: u32 = 0x00B8_D8FF;
/// A slider's place (right of a value's short text) and sprites: the track, the fill and its end piece.
pub const SLIDER_X: i32 = 556;
pub const SLIDER_WIDTH: i32 = 190;
pub const SLIDER_TRACK: u32 = 5;
pub const SLIDER_FILL: u32 = 3;
pub const SLIDER_END: u32 = 7;

/// The y of the text of row `n`.
pub fn row_y(n: usize) -> i32 {
    FIRST_ROW_Y + ROW_SPACING * n as i32
}

/// The live bar drawn: a dark track, and the part the axis fills.
fn live_bar<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    x: i32,
    y: i32,
    width: i32,
    at: Position,
) -> Result<(), S::Error> {
    s.solid([x, y, width, BAR_HEIGHT], TRACK_RGB)?;
    let (start, length) = at.span(width);
    if length > 0 {
        s.solid([x + start, y, length, BAR_HEIGHT], FILL_RGB)?;
    }
    if !at.pedal {
        s.solid([x + width / 2, y, 1, BAR_HEIGHT], MIDDLE_RGB)?;
    }
    Ok(())
}

/// A slider filled to `per_mille` of 1000 at `y`.
fn slider<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    y: i32,
    per_mille: u32,
) -> Result<(), S::Error> {
    s.sprite(SLIDER_TRACK, SLIDER_X, y)?;
    s.bar(Bar {
        x: SLIDER_X,
        y,
        width: SLIDER_WIDTH,
        fill_glyph: SLIDER_FILL,
        end_glyph: SLIDER_END,
        percent: (per_mille / 10) as i32,
    })
}

/// What the list shows.
#[derive(Clone, Debug)]
pub struct List {
    /// Where each axis the controller has reads now.
    pub axes: Vec<Position>,
    pub wheel: Vec<u8>,
    /// The full rotation, for a wheel.
    pub rotation: Option<Vec<u8>>,
}

/// The list drawn.
pub fn axes_list<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    list: &List,
) -> Result<(), S::Error> {
    for (n, at) in list.axes.iter().enumerate() {
        live_bar(s, VALUE_X, row_y(n) + 1, LIST_BAR_WIDTH, *at)?;
    }
    let wheel_y = row_y(list.axes.len());
    s.line(Font::Value, &list.wheel, VALUE_X, wheel_y)?;
    if let Some(rotation) = &list.rotation {
        s.line(Font::Value, rotation, VALUE_X, wheel_y + ROW_SPACING)?;
    }
    Ok(())
}

/// What one axis's page shows.
#[derive(Clone, Debug)]
pub struct AxisPage {
    /// The axis's name.
    pub axis: Vec<u8>,
    /// The settings' texts and levels (per mille of a slider), in `Column::ALL`'s order; a
    /// setting that is no level has none.
    pub values: [Vec<u8>; 4],
    pub levels: [Option<u32>; 4],
    pub at: Position,
}

/// The axis's page drawn.
pub fn axis_page<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    page: &AxisPage,
) -> Result<(), S::Error> {
    s.line(Font::Value, &page.axis, VALUE_X, row_y(0))?;
    for (n, _) in Column::ALL.iter().enumerate() {
        let y = row_y(n + 1);
        s.line(Font::Value, &page.values[n], VALUE_X, y)?;
        if let Some(level) = page.levels[n] {
            slider(s, y + 1, level)?;
        }
    }
    live_bar(
        s,
        VALUE_X,
        row_y(Column::ALL.len() + 1) + 1,
        PAGE_BAR_WIDTH,
        page.at,
    )
}
