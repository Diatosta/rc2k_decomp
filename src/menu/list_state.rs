//! The run-time state every list menu shares, whichever screen shows the list (the installer's
//! pages, the network lobby and session lists, the replay viewer): the highlighted item, the key
//! being repeated, the answer and the clocks that pace the highlight. One static.

use crate::memory::session::{PlainByte, PlainDword};

/// The list menu in use.
#[derive(rc2k_derive::Cells)]
pub struct ListState {
    /// The highlighted item.
    pub choice: PlainDword,
    /// The highlighted item before the last key.
    pub choice_before: PlainDword,
    /// How many times the held key has repeated.
    pub repeats: PlainDword,
    /// The last key taken (the replay viewer reads it too).
    pub last_key: PlainByte,
    /// A key was taken this frame.
    pub key_taken: PlainByte,
    /// The answer the list leaves.
    pub list_answer: PlainByte,
    /// The list is being left.
    pub list_leave: PlainByte,
    /// The clock of the last key.
    pub key_clock: PlainDword,
    /// The clock of the frame being drawn.
    pub frame_clock: PlainDword,
    /// The clock of the last move of the highlight.
    pub move_clock: PlainDword,
}

impl ListState {
    pub const fn new() -> Self {
        ListState {
            choice: PlainDword::new(0x0),
            choice_before: PlainDword::new(0x0),
            repeats: PlainDword::new(0x1),
            last_key: PlainByte::new(0x0),
            key_taken: PlainByte::new(0x0),
            list_answer: PlainByte::new(0x0),
            list_leave: PlainByte::new(0x0),
            key_clock: PlainDword::new(0x0),
            frame_clock: PlainDword::new(0x0),
            move_clock: PlainDword::new(0x0),
        }
    }
}
