//! The menu loop's flags, out of RAL.EXE's image (IDA 0x755525 .. 0x755595; phase 7, typed): the
//! item highlighted, the leaving flags, the text entry's counters, the sprite table and the menu
//! shown. The game keeps these dwords at addresses one past a multiple of four (IDA 0x755525 ..);
//! here they lie at the same distances from the start. A `#[repr(C)]` struct with the image's
//! layout, one static; `moved::MOVED` points the range at it. (Split 2026-10-25 from the front end's
//! block `menu::store`.)

use crate::session::{Bytes, Dword};

/// The menu loop's flags (IDA 0x755525 .. 0x755595).
#[repr(C)]
pub struct LoopFlags {
    /// `dword_755525`: The highlighted menu item, from 1.
    pub item: Dword,
    /// Unreferenced bytes (IDA 0x755529 .. 0x755551).
    pub _gap_755529: Bytes<40>,
    /// `dword_755551`: 1 after a long time without a key (`menu::update_blink`).
    pub idle: Dword,
    /// `dword_755555`: The main menu's frames without a key.
    pub idle_frames: Dword,
    /// Unreferenced bytes (IDA 0x755559 .. 0x75555D).
    pub _gap_755559: Bytes<4>,
    /// `dword_75555D`: 1 leaves the screen (`menu::screen_loop`).
    pub went_back: Dword,
    /// `dword_755561`: 1 leaves the screen at once; cleared by the loop.
    pub back: Dword,
    /// `dword_755565`: The event is over (`menu::leaving`).
    pub ended_b: Dword,
    /// `dword_755569`: Ask before leaving / done (`menu::leaving`).
    pub ended_a: Dword,
    /// `dword_75556D`: The characters typed so far (`menu::text_entry`).
    pub text_length: Dword,
    /// `dword_755571`: The most characters the text takes.
    pub text_max: Dword,
    /// `dword_755575`: Flipped by key 0x20 (`menu::handle_keys`).
    pub toggle: Dword,
    /// `g_champRow`: The championship row highlighted.
    pub champ_row: Dword,
    /// `g_optionsLoaded`: 1 once the options file has been read.
    pub options_loaded: Dword,
    /// `g_spriteTable`: The address of the menu sprites' glyph table.
    pub sprite_table: Dword,
    /// Unreferenced bytes (IDA 0x755585 .. 0x75558D).
    pub _gap_755585: Bytes<8>,
    /// `dword_75558D`: The current menu (the address of its head).
    pub menu: Dword,
    /// `dword_755591`: Not worked out.
    pub dword_755591: Dword,
}

/// First values the image's (`tests::loop_flags_starts_as_the_games`).
pub static LOOP_FLAGS: LoopFlags = LoopFlags {
    item: Dword::new(0x1),
    _gap_755529: Bytes::new([0; 40]),
    idle: Dword::new(0x0),
    idle_frames: Dword::new(0x0),
    _gap_755559: Bytes::new([0; 4]),
    went_back: Dword::new(0x0),
    back: Dword::new(0x0),
    ended_b: Dword::new(0x0),
    ended_a: Dword::new(0x0),
    text_length: Dword::new(0x0),
    text_max: Dword::new(0x1E),
    toggle: Dword::new(0x1),
    champ_row: Dword::new(0x0),
    options_loaded: Dword::new(0x0),
    sprite_table: Dword::new(0x0),
    _gap_755585: Bytes::new([0; 8]),
    menu: Dword::new(0x0),
    dword_755591: Dword::new(0x0),
};

/// IDA 0x755525, the first byte of the piece.
pub const START: usize = 0x755525;
/// IDA 0x755595, the end of the piece.
pub const END: usize = 0x755595;

/// Where IDA 0x755525 is now (`moved::MOVED`'s home).
pub fn loop_flags_addr() -> usize {
    crate::pointers::host_number(&LOOP_FLAGS as *const LoopFlags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn loop_flags_starts_as_the_games() {
        assert!(
            END - START <= size_of::<LoopFlags>()
                && size_of::<LoopFlags>() <= (END - START).next_multiple_of(4)
        );
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(loop_flags_addr()) as *const u8,
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(LoopFlags, item), 0x755525);
        at(offset_of!(LoopFlags, idle), 0x755551);
        at(offset_of!(LoopFlags, idle_frames), 0x755555);
        at(offset_of!(LoopFlags, went_back), 0x75555D);
        at(offset_of!(LoopFlags, back), 0x755561);
        at(offset_of!(LoopFlags, ended_b), 0x755565);
        at(offset_of!(LoopFlags, ended_a), 0x755569);
        at(offset_of!(LoopFlags, text_length), 0x75556D);
        at(offset_of!(LoopFlags, text_max), 0x755571);
        at(offset_of!(LoopFlags, toggle), 0x755575);
        at(offset_of!(LoopFlags, champ_row), 0x755579);
        at(offset_of!(LoopFlags, options_loaded), 0x75557D);
        at(offset_of!(LoopFlags, sprite_table), 0x755581);
        at(offset_of!(LoopFlags, menu), 0x75558D);
        at(offset_of!(LoopFlags, dword_755591), 0x755591);
    }
}
