//! The markers on the service area's four setup bars: which marker is drawn, and where on its
//! bar. Drawing it (the bar's length and the blit) is the caller's.

/// The bars' x (left, right) and y (upper, lower); 204 pixels long.
pub const LEFT_X: i32 = 0x81;
pub const RIGHT_X: i32 = 0x1D0;
pub(crate) const UPPER_Y: i32 = 0x199;
pub(crate) const LOWER_Y: i32 = 0x1D1;
pub const BAR_WIDTH: i32 = 0xCC;
/// The chosen value's marker (yellow) and the installed one's (orange).
pub(crate) const MARKER_GLYPH: u32 = 0x76;
pub(crate) const SECOND_MARKER_GLYPH: u32 = 0x77;

/// One of the six functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Marker {
    /// Always drawn on a lower bar, at the installed value (`SECOND_MARKER_GLYPH`).
    Always { x: i32 },
    /// On a lower bar at the chosen value: blinking while its item is edited, else drawn
    /// while the chosen value differs from the installed one or the item's repair flag is 1.
    Lower { item: u32, x: i32 },
    /// On an upper bar at the player's value, blinking while its item is edited.
    Upper { item: u32, x: i32 },
}

/// The menu's state the markers read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Menu {
    pub edit_mode: bool,
    pub item: u32,
    pub blink_on: bool,
}

/// What is to be done.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plan {
    /// Nothing drawn: the item.
    Hidden { item: u32 },
    /// The marker drawn: the percentage.
    Draw {
        x: i32,
        y: i32,
        glyph: u32,
        percent: i32,
    },
}

/// What a marker does. `percent` is the value it shows (the installed one for `Always`, the
/// chosen one for `Lower`, the player's for `Upper`), read only when needed; for `Lower`,
/// `installed` and `repair_flag` likewise.
pub fn plan<E>(
    marker: Marker,
    menu: Menu,
    percent: &dyn Fn() -> Result<i32, E>,
    installed: &dyn Fn() -> i32,
    repair_flag: &dyn Fn() -> u32,
) -> Result<(Plan, bool), E> {
    let editing = |item: u32| menu.edit_mode && menu.item == item;
    match marker {
        Marker::Always { x } => Ok((
            Plan::Draw {
                x,
                y: LOWER_Y,
                glyph: SECOND_MARKER_GLYPH,
                percent: percent()?,
            },
            false,
        )),
        Marker::Lower { item, x } => {
            let chosen = percent()?;
            let mut looked = false;
            if editing(item) {
                if !menu.blink_on {
                    return Ok((Plan::Hidden { item }, false));
                }
            } else if installed() == chosen {
                looked = true;
                if repair_flag() != 1 {
                    return Ok((Plan::Hidden { item }, true));
                }
            }
            Ok((
                Plan::Draw {
                    x,
                    y: LOWER_Y,
                    glyph: MARKER_GLYPH,
                    percent: chosen,
                },
                looked,
            ))
        }
        Marker::Upper { item, x } => {
            if editing(item) && !menu.blink_on {
                return Ok((Plan::Hidden { item }, false));
            }
            Ok((
                Plan::Draw {
                    x,
                    y: UPPER_Y,
                    glyph: MARKER_GLYPH,
                    percent: percent()?,
                },
                false,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers() {
        let ok = |v: i32| move || -> Result<i32, ()> { Ok(v) };
        let lower = Marker::Lower {
            item: 0x12,
            x: LEFT_X,
        };
        let menu = Menu {
            edit_mode: true,
            item: 0x12,
            blink_on: false,
        };
        assert_eq!(
            plan(lower, menu, &ok(40), &|| 40, &|| 0),
            Ok((Plan::Hidden { item: 0x12 }, false))
        );
        let menu = Menu {
            edit_mode: false,
            ..menu
        };
        assert_eq!(
            plan(lower, menu, &ok(40), &|| 40, &|| 0),
            Ok((Plan::Hidden { item: 0x12 }, true))
        );
        assert_eq!(
            plan(lower, menu, &ok(40), &|| 30, &|| 0),
            Ok((
                Plan::Draw {
                    x: LEFT_X,
                    y: LOWER_Y,
                    glyph: MARKER_GLYPH,
                    percent: 40
                },
                false
            ))
        );
        let upper = Marker::Upper {
            item: 0x10,
            x: RIGHT_X,
        };
        assert_eq!(plan(upper, menu, &|| Err(()), &|| 0, &|| 0), Err(()));
    }
}
