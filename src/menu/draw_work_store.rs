//! The drawing code's work variables, out of RAL.EXE's image (IDA 0x7559A4 .. 0x755A4C; phase 7,
//! typed): the blink, the scratch dwords the screens' routines pass values in (the original kept them
//! in globals: a button's width, x and y, the sorts' counters, the sliders' ends), the marker's
//! place, and the item action's value and the Escape flag. A `#[repr(C)]` struct with the image's
//! layout, one static; `moved::MOVED` points the range at it. (Split 2026-10-25 from the front end's
//! block `menu::store`.)

use crate::session::{Bytes, Dword, with_runs};

/// The drawing work variables (IDA 0x7559A4 .. 0x755A4C).
#[repr(C)]
pub struct DrawWork {
    /// `dword_7559A4`: 1 in the visible half of a blink (`menu::update_blink`).
    pub blink: Dword,
    /// `dword_7559A8`: 1 in the visible half of the text cursor's blink.
    pub cursor_on: Dword,
    /// `seconds`: A work variable: points, a button's width, the fill's end x (`gfx::blit_bar`).
    pub seconds: Dword,
    /// `dword_7559B0`: A work variable: a button's x.
    pub work_b0: Dword,
    /// `dword_7559B4`: A work variable: a button's y, the sorts' outer counter.
    pub work_b4: Dword,
    /// `dword_7559B8`: A work variable: the sorts' inner counter, a number's length.
    pub work_b8: Dword,
    /// `g_firstPlayerTime`: The first player's time.
    pub first_player_time: Dword,
    /// `glyph`: A work variable: a button's left glyph.
    pub glyph: Dword,
    /// `dword_7559C4`: A work variable: a button's right glyph.
    pub work_c4: Dword,
    /// `dword_7559C8`: Not worked out.
    pub dword_7559c8: Dword,
    /// `| Not worked out.`:
    pub dword_7559cc: Dword,
    /// `g_drawingHighlightedItem`: 1 while the highlighted item is drawn.
    pub drawing_highlighted_item: Dword,
    /// `g_overallFirstTime`: The overall first time.
    pub overall_first_time: Dword,
    /// `dword_7559D8`: A work variable: a button's middle.
    pub middle: Dword,
    /// `dword_7559DC`: The first slider's start (`screens::draw_controller_page`).
    pub slider_a_start: Dword,
    /// `dword_7559E0`: The first slider's end.
    pub slider_a_end: Dword,
    /// `dword_7559E4`: The second slider's start.
    pub slider_b_start: Dword,
    /// `dword_7559E8`: The second slider's end.
    pub slider_b_end: Dword,
    /// `dword_7559EC`: A work variable: a slot, a figure or a width (a font in IDA).
    pub figure: Dword,
    /// `hhmm`: A work variable: the hours and minutes, a gap, an x.
    pub hhmm: Dword,
    /// `dword_7559F4`: A work variable: a button's y.
    pub work_f4: Dword,
    /// `| Not worked out.`:
    pub work_f8: Dword,
    /// `| Not worked out.`:
    pub work_fc: Dword,
    /// `dword_755A00`: A work variable: a button's left glyph.
    pub work_00: Dword,
    /// `dword_755A04`: A work variable: a button's right glyph.
    pub work_04: Dword,
    /// `dword_755A08`: A work variable.
    pub work_08: Dword,
    /// `dword_755A0C`: A work variable: the device record the controller page draws, the saved-game slot picked, the checkpoints switch of the default times, `Leaving::field_a0c`.
    pub work_0c: Dword,
    /// Unreferenced bytes (IDA 0x755A10 .. 0x755A18).
    pub _gap_755a10: Bytes<8>,
    /// `g_markerX`: The marker's x (`gfx::blit_marker_pair`).
    pub marker_x: Dword,
    /// `g_markerY`: The marker's y.
    pub marker_y: Dword,
    /// `dword_755A20`: A marker's end (`screens::draw_highlighted_caption`).
    pub marker_end_a: Dword,
    /// `dword_755A24`: A marker's end.
    pub marker_end_b: Dword,
    /// `dword_755A28`: A marker's end.
    pub marker_end_c: Dword,
    /// `dword_755A2C`: A marker's end.
    pub marker_end_d: Dword,
    /// `dword_755A30`: Not worked out.
    pub dword_755a30: Dword,
    /// `| Not worked out.`:
    pub dword_755a34: Dword,
    /// `dword_755A38`: What the item's action leaves: 2 = stay on the screen as it is (`menu::screen_loop`).
    pub action_value: Dword,
    /// `dword_755A3C`: The screen was left with Escape.
    pub escaped: Dword,
    /// Unreferenced bytes (IDA 0x755A40 .. 0x755A4C).
    pub _gap_755a40: Bytes<12>,
}

/// First values the image's (`tests::draw_work_starts_as_the_games`).
pub static DRAW_WORK: DrawWork = DrawWork {
    blink: Dword::new(0x1),
    cursor_on: Dword::new(0x1),
    seconds: Dword::new(0x0),
    work_b0: Dword::new(0x0),
    work_b4: Dword::new(0x0),
    work_b8: Dword::new(0x0),
    first_player_time: Dword::new(0x0),
    glyph: Dword::new(0x0),
    work_c4: Dword::new(0x0),
    dword_7559c8: Dword::new(0x0),
    dword_7559cc: Dword::new(0x0),
    drawing_highlighted_item: Dword::new(0x0),
    overall_first_time: Dword::new(0x0),
    middle: Dword::new(0x0),
    slider_a_start: Dword::new(0x0),
    slider_a_end: Dword::new(0x0),
    slider_b_start: Dword::new(0x0),
    slider_b_end: Dword::new(0x0),
    figure: Dword::new(0x0),
    hhmm: Dword::new(0x0),
    work_f4: Dword::new(0x0),
    work_f8: Dword::new(0x0),
    work_fc: Dword::new(0x0),
    work_00: Dword::new(0x0),
    work_04: Dword::new(0x0),
    work_08: Dword::new(0x0),
    work_0c: Dword::new(0x0),
    _gap_755a10: Bytes::new([0; 8]),
    marker_x: Dword::new(0x0),
    marker_y: Dword::new(0x0),
    marker_end_a: Dword::new(0x0),
    marker_end_b: Dword::new(0x0),
    marker_end_c: Dword::new(0x0),
    marker_end_d: Dword::new(0x0),
    dword_755a30: Dword::new(0x0),
    dword_755a34: Dword::new(0x0),
    action_value: Dword::new(0x0),
    escaped: Dword::new(0x0),
    _gap_755a40: Bytes::new(with_runs::<12>(&[(0x0, &[0x86, 0x01])])),
};

/// IDA 0x7559A4, the first byte of the piece.
pub const START: usize = 0x7559A4;
/// IDA 0x755A4C, the end of the piece.
pub const END: usize = 0x755A4C;

/// Where IDA 0x7559A4 is now (`moved::MOVED`'s home).
pub fn draw_work_addr() -> u32 {
    crate::pointers::game_address(&DRAW_WORK as *const DrawWork)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn draw_work_starts_as_the_games() {
        assert!(
            END - START <= size_of::<DrawWork>()
                && size_of::<DrawWork>() <= (END - START).next_multiple_of(4)
        );
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(draw_work_addr()) as *const u8,
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(DrawWork, blink), 0x7559A4);
        at(offset_of!(DrawWork, cursor_on), 0x7559A8);
        at(offset_of!(DrawWork, seconds), 0x7559AC);
        at(offset_of!(DrawWork, work_b0), 0x7559B0);
        at(offset_of!(DrawWork, work_b4), 0x7559B4);
        at(offset_of!(DrawWork, work_b8), 0x7559B8);
        at(offset_of!(DrawWork, first_player_time), 0x7559BC);
        at(offset_of!(DrawWork, glyph), 0x7559C0);
        at(offset_of!(DrawWork, work_c4), 0x7559C4);
        at(offset_of!(DrawWork, dword_7559c8), 0x7559C8);
        at(offset_of!(DrawWork, dword_7559cc), 0x7559CC);
        at(offset_of!(DrawWork, drawing_highlighted_item), 0x7559D0);
        at(offset_of!(DrawWork, overall_first_time), 0x7559D4);
        at(offset_of!(DrawWork, middle), 0x7559D8);
        at(offset_of!(DrawWork, slider_a_start), 0x7559DC);
        at(offset_of!(DrawWork, slider_a_end), 0x7559E0);
        at(offset_of!(DrawWork, slider_b_start), 0x7559E4);
        at(offset_of!(DrawWork, slider_b_end), 0x7559E8);
        at(offset_of!(DrawWork, figure), 0x7559EC);
        at(offset_of!(DrawWork, hhmm), 0x7559F0);
        at(offset_of!(DrawWork, work_f4), 0x7559F4);
        at(offset_of!(DrawWork, work_f8), 0x7559F8);
        at(offset_of!(DrawWork, work_fc), 0x7559FC);
        at(offset_of!(DrawWork, work_00), 0x755A00);
        at(offset_of!(DrawWork, work_04), 0x755A04);
        at(offset_of!(DrawWork, work_08), 0x755A08);
        at(offset_of!(DrawWork, work_0c), 0x755A0C);
        at(offset_of!(DrawWork, marker_x), 0x755A18);
        at(offset_of!(DrawWork, marker_y), 0x755A1C);
        at(offset_of!(DrawWork, marker_end_a), 0x755A20);
        at(offset_of!(DrawWork, marker_end_b), 0x755A24);
        at(offset_of!(DrawWork, marker_end_c), 0x755A28);
        at(offset_of!(DrawWork, marker_end_d), 0x755A2C);
        at(offset_of!(DrawWork, dword_755a30), 0x755A30);
        at(offset_of!(DrawWork, dword_755a34), 0x755A34);
        at(offset_of!(DrawWork, action_value), 0x755A38);
        at(offset_of!(DrawWork, escaped), 0x755A3C);
    }
}
