//! The network game's screens: the lobby, the settings, the connections' and sessions' lists:
//! what is lit, where buttons and their frames go, which rows a list shows, how the highlight
//! moves, and what a value or a time is shown as.

use crate::Invalid;

/// Whether the blink is in its lit half: bit 3 of the time since the highlight moved clear.
pub fn blink_lit(frame_clock: u32, move_clock: u32) -> bool {
    frame_clock.wrapping_sub(move_clock) & 8 == 0
}

/// Whether item `item` is drawn lit: the highlighted one (`choice`), and being edited (the
/// item `edited`, asked only of the highlighted one) or in the blink's lit half (`blink`,
/// asked only then).
pub fn item_lit(
    item: u32,
    choice: u32,
    edited: impl FnOnce() -> u32,
    blink: impl FnOnce() -> bool,
) -> bool {
    item == choice && (item == edited() || blink())
}

/// A button's flags: centred on x.
pub const CENTRED: u32 = 1;
/// A button's flags: the dim font.
pub const DIM: u32 = 2;

/// The sprites of a button's frame: its left end, a 32 pixels' tile, its right end.
pub const LEFT_END: u32 = 0x11;
pub const TILE: u32 = 0x12;
pub const RIGHT_END: u32 = 0x13;
/// A frame's ends' width.
pub const END_WIDTH: u32 = 0x22;
/// A tile's width.
pub const TILE_WIDTH: u32 = 0x20;
/// A frame's width beyond its text's.
pub const FRAME_MORE: u32 = 0x44;

/// The tiles a frame takes for a text `width` wide: one per 32 pixels, at least one.
pub fn tiles(width: u32) -> u32 {
    let n = width.wrapping_add(0x1F) >> 5;
    if (n as i32) < 1 { 1 } else { n }
}

/// Where a button goes (`net::draw_button`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Button {
    /// Its frame's tiles.
    pub tiles: u32,
    /// Its frame's left end's x.
    pub left: u32,
    /// The x its text is drawn at.
    pub text_x: u32,
}

/// A button of a text `width` wide at `x` with `flags`: centred on x, or ending half its width
/// left of x with its text centred too.
pub fn button(width: u32, x: u32, flags: u32) -> Button {
    let span = width.wrapping_add(FRAME_MORE);
    let half = ((span as i32) >> 1) as u32;
    if flags & CENTRED != 0 {
        Button {
            tiles: tiles(width),
            left: x.wrapping_sub(span),
            text_x: x.wrapping_sub(half),
        }
    } else {
        Button {
            tiles: tiles(width),
            left: x.wrapping_sub(half),
            text_x: x,
        }
    }
}

/// The font a button's text is drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonFont {
    Plain,
    Lit,
    Dim,
}

/// A button's font: dim with `DIM`, else lit or plain.
pub fn button_font(flags: u32, lit: bool) -> ButtonFont {
    if flags & DIM != 0 {
        ButtonFont::Dim
    } else if lit {
        ButtonFont::Lit
    } else {
        ButtonFont::Plain
    }
}

/// The rows a list screen shows.
pub const LIST_ROWS: u32 = 16;
/// The connections' list's title.
pub const CONNECTIONS_TITLE: u32 = 0x14E;

/// The item list row `row` shows from the top `top`, if there is one (unsigned).
pub fn listed(row: u32, top: u32, count: u32) -> Option<u32> {
    let item = row.wrapping_add(top);
    (item < count).then_some(item)
}

/// The sessions' list's title: "choose a session", or "no sessions" with none listed.
pub fn sessions_title(sessions: u32) -> u32 {
    if sessions != 0 { 0x150 } else { 0x152 }
}

/// How a list's highlight moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKey {
    /// 0x66.
    Up,
    /// 0x6B.
    Down,
    /// 0x1C.
    Enter,
    /// 1.
    Escape,
    Other,
}

impl ListKey {
    pub fn of(key: u8) -> ListKey {
        match key {
            0x66 => ListKey::Up,
            0x6B => ListKey::Down,
            0x1C => ListKey::Enter,
            1 => ListKey::Escape,
            _ => ListKey::Other,
        }
    }
}

/// The highlight one up (not above 0), the top not below it: (the highlight, the top).
pub fn up(choice: u32, top: u32) -> (u32, u32) {
    let choice = (choice.wrapping_sub(1) as i32).max(0) as u32;
    let top = if (top as i32) > (choice as i32) {
        choice
    } else {
        top
    };
    (choice, top)
}

/// The highlight one down (not past the last of `count`), the top no more than fifteen
/// rows above it: (the highlight, the top).
pub fn down(choice: u32, top: u32, count: u32) -> (u32, u32) {
    let last = count.wrapping_sub(1);
    let mut choice = choice.wrapping_add(1);
    if (choice as i32) > (last as i32) {
        choice = last;
    }
    let least = choice.wrapping_sub(LIST_ROWS - 1);
    let top = if (top as i32) < (least as i32) {
        least
    } else {
        top
    };
    (choice, top)
}

/// Whether the list's top moves with the highlighted session found again after a new
/// listing: with more than sixteen listed (else it is 0).
pub fn keeps_top(sessions: u32) -> bool {
    sessions > LIST_ROWS
}

/// The list's top once the highlighted session (`choice`) is found again at `found`: moved
/// by as much (signed, not below 0).
pub fn moved_top(found: u32, top: u32, choice: u32) -> u32 {
    let top = top.wrapping_sub(choice).wrapping_add(found);
    if (top as i32) < 0 { 0 } else { top }
}

/// The settings screen's last item's frame and text, right to left from this x.
pub const LAST_ITEM_RIGHT: u32 = 0x311;

/// The settings screen's last item (a text `width` wide): its tiles and its text's x.
/// Refused: a text of no width.
pub fn last_item(width: u32) -> Result<(u32, u32), Invalid> {
    let tiles = width.wrapping_add(0x1F) >> 5;
    if tiles == 0 {
        return Err(Invalid(
            "a last item of no width: the frame's loop never ends",
        ));
    }
    let free = (tiles << 5).wrapping_add(FRAME_MORE).wrapping_sub(width);
    let half = ((free as i32) >> 1) as u32;
    Ok((tiles, LAST_ITEM_RIGHT.wrapping_sub(half)))
}

/// A lobby button's flags: centred; dim while this player is ready (`state` bit 0) for all
/// but the chat (item 3), and for "ready" (item 0) back from a race.
pub fn lobby_button_flags(
    state: crate::net::players::State,
    item: u32,
    back_from_race: u32,
) -> u32 {
    let mut flags = CENTRED;
    if state.is_ready() && item < 3 {
        flags |= DIM;
    }
    if back_from_race == 1 && item == 0 {
        flags |= DIM;
    }
    flags
}

/// The x of the lobby's next button left: past this one (its text `width` wide) and 8
/// more.
pub fn next_button_x(x: u32, width: u32) -> u32 {
    x.wrapping_sub(width.wrapping_add(FRAME_MORE))
        .wrapping_sub(8)
}

/// The lobby's bottom row of buttons right to left: (the string, shown), items 0 .. 3: ready,
/// the car and the stage when they can be chosen (their switches 1), and the chat.
pub fn lobby_buttons(car_item: u32, stage_item: u32) -> [(u32, bool); 4] {
    [
        (0x15D, true),
        (0x15B, car_item == 1),
        (0x15C, stage_item == 1),
        (0x15E, true),
    ]
}

/// The first item of the upper row of buttons, above the bottom row: the settings, then (when
/// the last network race's replay is kept) the button that saves it.
pub const UPPER_ROW_FIRST: u32 = 4;
/// The upper row's rightmost button: the in-race options.
pub const SETTINGS_ITEM: u32 = 4;
/// The upper row's button that saves the last race's replay.
pub const SAVE_REPLAY_ITEM: u32 = 5;

/// The upper row's buttons right to left: (the string, shown), items 4 and 5.
pub fn upper_buttons(replay_kept: bool) -> [(u32, bool); 2] {
    [
        (crate::strings::own::id::LOBBY_SETTINGS, true),
        (crate::strings::own::id::SAVE_REPLAY, replay_kept),
    ]
}

/// The item an up key (`up`) or a down key moves to from `item`: from the bottom row up to the
/// settings button, from the upper row down to the chat; no move in the row it
/// would leave for.
pub fn other_row_item(item: u32, up: bool) -> u32 {
    match (item >= UPPER_ROW_FIRST, up) {
        (false, true) => SETTINGS_ITEM,
        (true, false) => 3,
        _ => item,
    }
}

/// An item of the upper row moved one button: `to_next` toward the next item (the left), else
/// back; held to the shown buttons.
pub fn upper_row_step(item: u32, to_next: bool, replay_kept: bool) -> u32 {
    let last = if replay_kept {
        SAVE_REPLAY_ITEM
    } else {
        SETTINGS_ITEM
    };
    if to_next {
        (item + 1).min(last)
    } else {
        item.saturating_sub(1).max(UPPER_ROW_FIRST)
    }
}

/// The furthest a time may be behind the first row's to be shown as a gap with a '+'.
pub const MOST_GAP: u32 = 0x36_EE7F;

/// A lobby row's time shown: the value written, and whether a '+' goes over its first
/// character. With gaps on, a time not the first row's is its difference from it, a '+'
/// when within `MOST_GAP` (unsigned).
pub fn shown_time(time: u32, first: u32, gaps: bool) -> (u32, bool) {
    if gaps && time != first {
        let gap = time.wrapping_sub(first);
        (gap, gap <= MOST_GAP)
    } else {
        (time, false)
    }
}

/// A nationality's flag's sprite.
pub fn flag_sprite(nationality: u8) -> u32 {
    nationality as u32 + 0x1C
}

/// A car's sprite in the lobby (cars 0x18 and 0x19 swapped).
pub fn car_sprite(car: u8) -> u32 {
    let car = if (0x18..=0x19).contains(&car) {
        car ^ 1
    } else {
        car
    };
    car as u32 + 0x44
}

/// The string shown for a value or a time there is none of.
pub const NOTHING: u32 = 0x16E;
/// The string shown for a race with no time.
pub const NO_TIME: u32 = 0x161;
/// The caret's string after a text being edited.
pub const CARET: u32 = 4;

/// What a setting's value is shown as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value {
    /// The item being edited: its text and the caret.
    Edited,
    /// An empty text: `NOTHING`.
    Nothing,
    /// The text as it is.
    AsIs,
}

/// A text value (`first` its first byte; read only when not edited).
pub fn text_value(edited: bool, first: impl FnOnce() -> u8) -> Value {
    if edited {
        Value::Edited
    } else if first() == 0 {
        Value::Nothing
    } else {
        Value::AsIs
    }
}

/// What a number setting is written as before it is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Number {
    /// The item being edited: the number's text as typed.
    Typed,
    /// 0: `NOTHING`.
    Nothing,
    /// The number in figures.
    Figures,
}

pub fn number_value(edited: bool, number: u32) -> Number {
    if edited {
        Number::Typed
    } else if number == 0 {
        Number::Nothing
    } else {
        Number::Figures
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::players::State;

    #[test]
    fn lit() {
        assert!(blink_lit(0x1000, 0x1000));
        assert!(!blink_lit(0x1008, 0x1000));
        assert!(blink_lit(0x1010, 0x1000));
        assert!(item_lit(1, 1, || u32::MAX, || true));
        assert!(!item_lit(1, 1, || u32::MAX, || false));
        assert!(item_lit(1, 1, || 1, || unreachable!()));
        assert!(!item_lit(2, 1, || unreachable!(), || unreachable!()));
    }

    #[test]
    fn buttons() {
        assert_eq!(tiles(0), 1);
        assert_eq!(tiles(0x20), 1);
        assert_eq!(tiles(0x21), 2);
        assert_eq!(tiles(0xFFFF_FFF0), 1);
        assert_eq!(
            button(0x40, 0x310, 0),
            Button {
                tiles: 2,
                left: 0x310 - 0x42,
                text_x: 0x310
            }
        );
        assert_eq!(
            button(0x40, 0x310, 1),
            Button {
                tiles: 2,
                left: 0x310 - 0x84,
                text_x: 0x310 - 0x42
            }
        );
        assert_eq!(button(0xFFFF_FFF0, 0, 0).left, 0xFFFF_FFE6);
        assert_eq!(button_font(2, true), ButtonFont::Dim);
        assert_eq!(button_font(1, true), ButtonFont::Lit);
        assert_eq!(button_font(0, false), ButtonFont::Plain);
        assert_eq!(lobby_button_flags(State::READY, 0, 0), 3);
        assert_eq!(lobby_button_flags(State::READY, 3, 0), 1);
        assert_eq!(lobby_button_flags(State::default(), 0, 1), 3);
        assert_eq!(lobby_button_flags(State::default(), 1, 1), 1);
        assert_eq!(next_button_x(0x310, 0x40), 0x310 - 0x84 - 8);
        assert_eq!(lobby_buttons(1, 0)[2], (0x15C, false));
        assert!(!upper_buttons(false)[1].1 && upper_buttons(true)[1].1);
        assert!(upper_buttons(false)[0].1);
        assert_eq!(other_row_item(3, true), SETTINGS_ITEM);
        assert_eq!(other_row_item(0, false), 0);
        assert_eq!(other_row_item(5, false), 3);
        assert_eq!(upper_row_step(4, true, false), 4);
        assert_eq!(upper_row_step(4, true, true), 5);
        assert_eq!(upper_row_step(5, false, true), 4);
        assert_eq!(upper_row_step(4, false, true), 4);
        assert_eq!(
            last_item(0),
            Err(Invalid(
                "a last item of no width: the frame's loop never ends"
            ))
        );
        assert_eq!(last_item(1), Ok((1, 0x311 - 0x31)));
        assert_eq!(last_item(0x45), Ok((3, 0x311 - 0x2F)));
    }

    #[test]
    fn lists() {
        assert_eq!(listed(3, 4, 20), Some(7));
        assert_eq!(listed(3, 4, 7), None);
        assert_eq!(sessions_title(0), 0x152);
        assert_eq!(sessions_title(2), 0x150);
        assert_eq!(ListKey::of(0x6B), ListKey::Down);
        assert_eq!(up(0, 0), (0, 0));
        assert_eq!(up(5, 5), (4, 4));
        assert_eq!(up(5, 2), (4, 2));
        assert_eq!(down(2, 0, 3), (2, 0));
        assert_eq!(down(16, 0, 30), (17, 2));
        assert_eq!(down(0, 0, 0), (u32::MAX, 0));
        assert!(!keeps_top(16) && keeps_top(17));
        assert_eq!(moved_top(9, 3, 4), 8);
        assert_eq!(moved_top(0, 0, 4), 0);
    }

    #[test]
    fn values() {
        assert_eq!(shown_time(500, 500, true), (500, false));
        assert_eq!(shown_time(700, 500, true), (200, true));
        assert_eq!(
            shown_time(500, 700, true),
            (500u32.wrapping_sub(700), false)
        );
        assert_eq!(shown_time(700, 500, false), (700, false));
        assert_eq!(car_sprite(0x18), 0x19 + 0x44);
        assert_eq!(car_sprite(0x19), 0x18 + 0x44);
        assert_eq!(car_sprite(3), 0x47);
        assert_eq!(flag_sprite(2), 0x1E);
        assert_eq!(text_value(true, || unreachable!()), Value::Edited);
        assert_eq!(text_value(false, || 0), Value::Nothing);
        assert_eq!(text_value(false, || b'a'), Value::AsIs);
        assert_eq!(number_value(false, 0), Number::Nothing);
        assert_eq!(number_value(false, 4), Number::Figures);
        assert_eq!(number_value(true, 0), Number::Typed);
    }
}
