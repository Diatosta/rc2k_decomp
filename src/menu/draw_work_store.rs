//! The drawing code's work variables, out of RAL.EXE's image (IDA 0x7559A4 .. 0x755A4C): the blink, the scratch dwords the screens' routines pass values in (the original kept them
//! in globals: a button's width, x and y, the sorts' counters, the sliders' ends), the marker's
//! place, and the item action's value and the Escape flag. A `#[repr(C)]` struct with the image's
//! layout, one static.

use crate::session::{PlainDword, PlainFlag};

/// The drawing work variables (IDA 0x7559A4 .. 0x755A4C).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct DrawWork {
    /// `dword_7559A4`: 1 in the visible half of a blink (`menu::update_blink`).
    pub blink: PlainFlag,
    /// `dword_7559A8`: 1 in the visible half of the text cursor's blink.
    pub cursor_on: PlainFlag,
    /// `seconds`: A work variable: points, a button's width, the fill's end x (`gfx::blit_bar`).
    pub seconds: PlainDword,
    /// (0x7559B0): a scratch variable the screens' routines pass values in (the original used a global where a register would do): a button's x, a row's y, a sprite's y, a sum of times (`saves::default_times`).
    pub scratch_b0: PlainDword,
    /// (0x7559B4): scratch: a button's y, the sorts' outer counter (SORTED - 1), a car glyph, a stage, a player-results extra glyph.
    pub scratch_b4: PlainDword,
    /// (0x7559B8): scratch: the sorts' inner counter (SORTED), a number's length (`draw_spaced_number`), a gap, a time.
    pub scratch_b8: PlainDword,
    /// `g_firstPlayerTime`: The first player's time.
    pub first_player_time: PlainDword,
    /// `glyph`: A work variable: a button's left glyph.
    pub glyph: PlainDword,
    /// (0x7559C4): scratch: a button's right glyph (`screens::buttons`, `game_screen`).
    pub scratch_c4: PlainDword,
    /// `dword_7559C8`: Not worked out.
    pub dword_7559c8: PlainDword,
    /// `| Not worked out.`:
    pub dword_7559cc: PlainDword,
    /// `g_drawingHighlightedItem`: 1 while the highlighted item is drawn.
    pub drawing_highlighted_item: PlainDword,
    /// `g_overallFirstTime`: The overall first time.
    pub overall_first_time: PlainDword,
    /// `dword_7559D8`: A work variable: a button's middle.
    pub middle: PlainDword,
    /// `dword_7559DC`: The first slider's start (`screens::draw_controller_page`).
    pub slider_a_start: PlainDword,
    /// `dword_7559E0`: The first slider's end.
    pub slider_a_end: PlainDword,
    /// `dword_7559E4`: The second slider's start.
    pub slider_b_start: PlainDword,
    /// `dword_7559E8`: The second slider's end.
    pub slider_b_end: PlainDword,
    /// `dword_7559EC`: A work variable: a slot, a figure or a width (a font in IDA).
    pub figure: PlainDword,
    /// `hhmm`: A work variable: the hours and minutes, a gap, an x.
    pub hhmm: PlainDword,
    /// (0x7559F4): scratch: the second kind of button's y (`screens::buttons`).
    pub scratch_f4: PlainDword,
    /// `| Not worked out.`:
    pub work_f8: PlainDword,
    /// `| Not worked out.`:
    pub work_fc: PlainDword,
    /// (0x755A00): scratch: the second kind of button's left glyph (`screens::buttons`).
    pub scratch_00: PlainDword,
    /// (0x755A04): scratch: the second kind of button's right glyph (`screens::buttons`).
    pub scratch_04: PlainDword,
    /// (0x755A08): scratch: a service row's string (`game_screen`, the service page).
    pub scratch_08: PlainDword,
    /// (0x755A0C): scratch: the device record the controller page draws, the saved-game slot picked (`save_replay_loop` returns it), the checkpoints switch of the default times (1: guess them; `set_up_front_end`), a service row, a record, and `Leaving`'s last word (-1 on going back).
    pub scratch_0c: PlainDword,
    /// `g_markerX`: The marker's x (`gfx::blit_marker_pair`).
    pub marker_x: PlainDword,
    /// `g_markerY`: The marker's y.
    pub marker_y: PlainDword,
    /// `dword_755A20`: A marker's end (`screens::draw_highlighted_caption`).
    pub marker_end_a: PlainDword,
    /// `dword_755A24`: A marker's end.
    pub marker_end_b: PlainDword,
    /// `dword_755A28`: A marker's end.
    pub marker_end_c: PlainDword,
    /// `dword_755A2C`: A marker's end.
    pub marker_end_d: PlainDword,
    /// `dword_755A30`: Not worked out.
    pub dword_755a30: PlainDword,
    /// `| Not worked out.`:
    pub dword_755a34: PlainDword,
    /// `dword_755A38`: What the item's action leaves: 2 = stay on the screen as it is (`menu::screen_loop`).
    pub action_mode: PlainDword,
    /// `dword_755A3C`: The screen was left with Escape.
    pub escape_pressed: PlainFlag,
}

impl DrawWork {
    pub const fn new() -> Self {
        DrawWork {
            blink: PlainFlag::new(true),
            cursor_on: PlainFlag::new(true),
            seconds: PlainDword::new(0x0),
            scratch_b0: PlainDword::new(0x0),
            scratch_b4: PlainDword::new(0x0),
            scratch_b8: PlainDword::new(0x0),
            first_player_time: PlainDword::new(0x0),
            glyph: PlainDword::new(0x0),
            scratch_c4: PlainDword::new(0x0),
            dword_7559c8: PlainDword::new(0x0),
            dword_7559cc: PlainDword::new(0x0),
            drawing_highlighted_item: PlainDword::new(0x0),
            overall_first_time: PlainDword::new(0x0),
            middle: PlainDword::new(0x0),
            slider_a_start: PlainDword::new(0x0),
            slider_a_end: PlainDword::new(0x0),
            slider_b_start: PlainDword::new(0x0),
            slider_b_end: PlainDword::new(0x0),
            figure: PlainDword::new(0x0),
            hhmm: PlainDword::new(0x0),
            scratch_f4: PlainDword::new(0x0),
            work_f8: PlainDword::new(0x0),
            work_fc: PlainDword::new(0x0),
            scratch_00: PlainDword::new(0x0),
            scratch_04: PlainDword::new(0x0),
            scratch_08: PlainDword::new(0x0),
            scratch_0c: PlainDword::new(0x0),
            marker_x: PlainDword::new(0x0),
            marker_y: PlainDword::new(0x0),
            marker_end_a: PlainDword::new(0x0),
            marker_end_b: PlainDword::new(0x0),
            marker_end_c: PlainDword::new(0x0),
            marker_end_d: PlainDword::new(0x0),
            dword_755a30: PlainDword::new(0x0),
            dword_755a34: PlainDword::new(0x0),
            action_mode: PlainDword::new(0x0),
            escape_pressed: PlainFlag::new(false),
        }
    }
}
