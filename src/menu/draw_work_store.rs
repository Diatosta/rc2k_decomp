//! The drawing code's work variables: the blink, the scratch dwords the screens' routines pass
//! values in (a button's width, x and y, the sorts' counters, the sliders' ends), the marker's
//! place, and the item action's value and the Escape flag. One static.

use crate::session::{PlainDword, PlainFlag};

/// The drawing work variables.
#[derive(rc2k_derive::Cells)]
pub struct DrawWork {
    /// 1 in the visible half of a blink (`menu::update_blink`).
    pub blink: PlainFlag,
    /// 1 in the visible half of the text cursor's blink.
    pub cursor_on: PlainFlag,
    /// `seconds`: A work variable: points, a button's width, the fill's end x (`gfx::blit_bar`).
    pub seconds: PlainDword,
    /// A scratch variable the screens' routines pass values in: a button's x, a row's y, a sprite's
    /// y, a sum of times (`saves::default_times`).
    pub scratch_b0: PlainDword,
    /// Scratch: a button's y, the sorts' outer counter (SORTED - 1), a car glyph, a stage, a
    /// player-results extra glyph.
    pub scratch_b4: PlainDword,
    /// Scratch: the sorts' inner counter (SORTED), a number's length (`draw_spaced_number`), a gap,
    /// a time.
    pub scratch_b8: PlainDword,
    /// The first player's time.
    pub first_player_time: PlainDword,
    /// `glyph`: A work variable: a button's left glyph.
    pub glyph: PlainDword,
    /// Scratch: a button's right glyph (`screens::buttons`, `game_screen`).
    pub scratch_c4: PlainDword,
    pub dword_7559c8: PlainDword,
    /// `| Not worked out.`:
    pub dword_7559cc: PlainDword,
    /// 1 while the highlighted item is drawn.
    pub drawing_highlighted_item: PlainDword,
    /// The overall first time.
    pub overall_first_time: PlainDword,
    /// A work variable: a button's middle.
    pub middle: PlainDword,
    /// The first slider's start (`screens::draw_controller_page`).
    pub slider_a_start: PlainDword,
    /// The first slider's end.
    pub slider_a_end: PlainDword,
    /// The second slider's start.
    pub slider_b_start: PlainDword,
    /// The second slider's end.
    pub slider_b_end: PlainDword,
    /// A work variable: a slot, a figure or a width.
    pub figure: PlainDword,
    /// `hhmm`: A work variable: the hours and minutes, a gap, an x.
    pub hhmm: PlainDword,
    /// Scratch: the second kind of button's y (`screens::buttons`).
    pub scratch_f4: PlainDword,
    /// `| Not worked out.`:
    pub work_f8: PlainDword,
    /// `| Not worked out.`:
    pub work_fc: PlainDword,
    /// Scratch: the second kind of button's left glyph (`screens::buttons`).
    pub scratch_00: PlainDword,
    /// Scratch: the second kind of button's right glyph (`screens::buttons`).
    pub scratch_04: PlainDword,
    /// Scratch: a service row's string (`game_screen`, the service page).
    pub scratch_08: PlainDword,
    /// Scratch: the device record the controller page draws, the saved-game slot picked
    /// (`save_replay_loop` returns it), the checkpoints switch of the default times (1: guess them;
    /// `set_up_front_end`), a service row, a record, and `Leaving`'s last word (-1 on going back).
    pub scratch_0c: PlainDword,
    /// The marker's x (`gfx::blit_marker_pair`).
    pub marker_x: PlainDword,
    /// The marker's y.
    pub marker_y: PlainDword,
    /// A marker's end (`screens::draw_highlighted_caption`).
    pub marker_end_a: PlainDword,
    /// A marker's end.
    pub marker_end_b: PlainDword,
    /// A marker's end.
    pub marker_end_c: PlainDword,
    /// A marker's end.
    pub marker_end_d: PlainDword,
    pub dword_755a30: PlainDword,
    /// `| Not worked out.`:
    pub dword_755a34: PlainDword,
    /// What the item's action leaves: 2 = stay on the screen as it is (`menu::screen_loop`).
    pub action_mode: PlainDword,
    /// The screen was left with Escape.
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
