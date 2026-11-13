//! The screens that draw one string: the leave question, string 0x1FD, the arcade stage's
//! title and the no-controller note.

use crate::screens::screen::{Drawn, Screen};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    Question,
    Value,
}

/// These screens keep nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {}

/// A string and where it goes.
#[derive(Clone, Copy, Debug)]
pub struct Note {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub font: Font,
}

/// The "leave?" question.
pub const LEAVE_QUESTION: Note = Note {
    id: 0x537,
    x: 0x197,
    y: 0x122,
    font: Font::Question,
};
/// String 0x1FD (`draw_note_1fd`).
pub const NOTE_1FD: Note = Note {
    id: 0x1FD,
    x: 0x19A,
    y: 0x118,
    font: Font::Question,
};
/// The arcade's after-stage screen's title.
pub const ARCADE_STAGE_TITLE: Note = Note {
    id: 0x1FC,
    x: 0x19A,
    y: 0x118,
    font: Font::Question,
};
/// The controller page's "none".
pub const NO_CONTROLLER: Note = Note {
    id: 0x56D,
    x: 0x1BC,
    y: 0xD5,
    font: Font::Value,
};

/// Draws the note.
pub fn note<S: Screen<Font = Font, Work = Work>>(s: &mut S, n: &Note) -> Result<Drawn, S::Error> {
    s.string(n.font, n.id, n.x, n.y)
}
