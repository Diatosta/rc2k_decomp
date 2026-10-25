//! The replay viewer's screen (IDA 0x5C291C .. 0x5C2FE4; phase 7, typed): the cameras' text ids, the bar, buttons, marks and place rectangles (scratch ones written each frame), the quad scale floats, the picture switches, the replay patch switches kept, the frame-picture recording state (three dwords at odd addresses), the NET and display text strings between 0x5C2CF7 and 0x5C2F81 (left as unknown bytes), the view marks' state and the listener place and turn. Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
pub struct ViewerPanelBlock {
    /// +0x0 (0x5C291C): not named.
    pub unknown_0: [Dword; 24],
    /// +0x60 (0x5C297C): `dword_5C297C`: rectangle (0,0,800,32) of the viewer bar's frame (`race::viewer_frame`).
    pub bar_frame_rect: [Dword; 4],
    /// +0x70 (0x5C298C): `dword_5C298C`: first button rectangle (`race::viewer_frame`).
    pub button_rect_a: [Dword; 4],
    /// +0x80 (0x5C299C): `dword_5C299C`: second button rectangle (`race::viewer_frame`).
    pub button_rect_b: [Dword; 4],
    /// +0x90 (0x5C29AC): `dword_5C29AC`: third button rectangle (`race::viewer_frame`).
    pub button_rect_c: [Dword; 4],
    /// +0xa0 (0x5C29BC): `dword_5C29BC`: fourth button rectangle (`race::viewer_frame`).
    pub button_rect_d: [Dword; 4],
    /// +0xb0 (0x5C29CC): `dword_5C29CC`: the bar's rectangle (x, y, w at +8, h) (`race::viewer_frame`).
    pub bar_rect: [Dword; 4],
    /// +0xc0 (0x5C29DC): `dword_5C29DC`: a mark's rectangle, x written each frame (`race::viewer_frame`).
    pub mark_rect: [Dword; 4],
    /// +0xd0 (0x5C29EC): `dword_5C29EC`: the place marker's rectangle, x written each frame (`race::viewer_frame`).
    pub place_rect: [Dword; 4],
    /// +0xe0 (0x5C29FC): `dword_5C29FC`: the item quad's place, zero in the image (`race::draw_viewer_item`).
    pub item_place: [Dword; 4],
    /// +0xf0 (0x5C2A0C): `dword_5C2A0C`: the quad's screen rectangle, scratch, zero in the image (`race::draw_viewer_panel`, `drive::tyre_screen_parts`).
    pub screen_rect: [Dword; 4],
    /// +0x100 (0x5C2A1C): `dword_5C2A1C`: the quad's texture rectangle, scratch, zero in the image (`race::draw_viewer_panel`, `drive::tyre_screen_parts`, `race::draw_tyre_change`).
    pub texture_rect: [Dword; 4],
    /// +0x110 (0x5C2A2C): not named.
    pub unknown_110: [Dword; 40],
    /// +0x1b0 (0x5C2ACC): `dword_5C2ACC`: texture rectangle (249,0,7,29) of a mark (`race::viewer_frame`).
    pub mark_picture_rect: [Dword; 4],
    /// +0x1c0 (0x5C2ADC): not named.
    pub unknown_1c0: [Dword; 4],
    /// +0x1d0 (0x5C2AEC): `dword_5C2AEC`: texture rectangle (48,48,48,48) of the place marker (`race::viewer_frame`).
    pub place_picture_rect: [Dword; 4],
    /// +0x1e0 (0x5C2AFC): not named.
    pub unknown_1e0: [Dword; 76],
    /// +0x310 (0x5C2C2C): `dword_5C2C2C`: texture rectangle (0,240,16,16) of a button end (`race::viewer_frame`).
    pub button_picture_rect: [Dword; 4],
    /// +0x320 (0x5C2C3C): `dword_5C2C3C`: texture rectangle (15,240,241,16) of a button body (`race::viewer_frame`).
    pub button_body_rect: [Dword; 4],
    /// +0x330 (0x5C2C4C): `dword_5C2C4C` (20 bytes in IDA): texture rectangle (1,241,14,14) of the bar's frame, plus 0.5f as its fifth dword (`race::viewer_frame`).
    pub frame_picture_rect: [Dword; 5],
    /// +0x344 (0x5C2C60): `flt_5C2C60`: 1.0 (`race::tyre_change_gauge`).
    pub one: Float,
    /// +0x348 (0x5C2C64): `flt_5C2C64`: 1/256 (`drive::tyre_quad`, `drive::tyre_turned_quad`).
    pub texel_scale: Float,
    /// +0x34c (0x5C2C68): `flt_5C2C68`: 0.00125 (a reading) (`drive::tyre_quad`, `drive::tyre_turned_quad`).
    pub texel_scale_small: Float,
    /// +0x350 (0x5C2C6C): `flt_5C2C6C`: 0.00167 (a reading) (`drive::tyre_quad`, `drive::tyre_turned_quad`).
    pub texel_scale_smaller: Float,
    /// +0x354 (0x5C2C70): `off_5C2C70`: two relocated pointers to texture rectangles, a camera byte 2 or 3 picks one (`race::draw_viewer_item`).
    pub switch_pictures_a: [Dword; 2],
    /// +0x35c (0x5C2C78): `off_5C2C78`: four relocated pointers to texture rectangles, by camera byte 5 (`race::draw_viewer_item`).
    pub switch_pictures_b: [Dword; 4],
    /// +0x36c (0x5C2C88): `off_5C2C88`: three relocated pointers to texture rectangles, by camera byte 4 (`race::draw_viewer_item`).
    pub switch_pictures_c: [Dword; 3],
    /// +0x378 (0x5C2C94): `dword_5C2C94`: the replay's physics patches are in force (`race::replay_patches`).
    pub patches_in_force: Dword,
    /// +0x37c (0x5C2C98): `unk_5C2C98`: the game's 13 patch switches kept while a replay's are in force (`race::replay_patches`).
    pub kept_patch_switches: [Dword; 13],
    /// +0x3b0 (0x5C2CCC): `g_replayRecording`: the frames are saved as pictures (`race::viewer`, `race::replay_screenshot`).
    pub replay_recording: Dword,
    /// +0x3b4 (0x5C2CD0): `dword_5C2CD0`: the picture clock (ms), also read by `system::tick_count` (`race::replay_screenshot`).
    pub picture_clock: Dword,
    /// +0x3b8 (0x5C2CD4): `dword_5C2CD4`: pictures started; `system::tick_count` overrides its clock when 1 (`race::replay_screenshot`).
    pub pictures_started: Dword,
    /// +0x3bc (0x5C2CD8): `off_5C2CD8`: the text rpl and a NUL, the pictures' folder (`race::start_frame_pictures`).
    pub picture_folder: Dword,
    /// +0x3c0 (0x5C2CDC): `aRpl000000Pcx`: the 15-byte name rpl\000000.pcx, taken in place; the last 3 bytes (0x5C2CE8 .. 0x5C2CEB) are not a whole dword (`race::replay_screenshot`).
    pub picture_name: [Dword; 3],
    /// +0x3cc (0x5C2CE8): not named.
    pub unknown_3cc: [Dword; 167],
    /// +0x668 (0x5C2F84): `asc_5C2F84`: a 2-byte tab text, a pointer taken in place by `marks::init_mark_step` (a reading).
    pub settings_text: Dword,
    /// +0x66c (0x5C2F88): `dword_5C2F88`: set by `drive::finish_timing`, cleared twice by `drive::finish_race` (a reading).
    pub race_done: Dword,
    /// +0x670 (0x5C2F8C): `dword_5C2F8C`: the view's marks layout (`race::frame_views`).
    pub marks_layout: Dword,
    /// +0x674 (0x5C2F90): `dword_5C2F90`: this view's marks show (`race::frame_views`).
    pub own_marks_shown: Dword,
    /// +0x678 (0x5C2F94): `dword_5C2F94`: the other view's marks show; `drive::race_init` sets it (`race::frame_views`).
    pub other_marks_shown: Dword,
    /// +0x67c (0x5C2F98): not named.
    pub unknown_67c: [Dword; 2],
    /// +0x684 (0x5C2FA0): `dword_5C2FA0` ..: the view's place negated, three whole numbers (`drive::set_listener`).
    pub listener_place: [Dword; 3],
    /// +0x690 (0x5C2FAC): `dword_5C2FAC` ..: the view's turn matrix, nine floats, popped from the x87 (image value identity) (`drive::set_listener`).
    pub listener_turn: [Float; 9],
    /// +0x6b4 (0x5C2FD0): `dword_5C2FD0`: `dword_7558DC` copied after the loading picture (`race::single_rally`).
    pub after_loading_copy: Dword,
    /// +0x6b8 (0x5C2FD4): `dword_5C2FD4`: the viewed car's +0xABC >= 0x10; image value 1 (`drive::set_listener`).
    pub listener_near: Dword,
    /// +0x6bc (0x5C2FD8): not named.
    pub unknown_6bc: [Dword; 2],
    /// +0x6c4 (0x5C2FE0): `dword_5C2FE0`: a copy of the race's cars (`marks::init_mark_step`).
    pub cars_copy: Dword,
}

/// The piece: the race block's bytes at IDA 0x5C291C seen as `ViewerPanelBlock`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = ViewerPanelBlock;
    fn deref(&self) -> &ViewerPanelBlock {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(piece_addr() as *const ViewerPanelBlock) }
    }
}

pub static VIEWER_PANEL: InRaceBlock = InRaceBlock;

/// IDA 0x5C291C, the first byte of the piece.
pub const START: usize = 0x5C291C;
/// IDA 0x5C2FE4, the end of the piece.
pub const END: usize = 0x5C2FE4;

const _: () = assert!(END - START == core::mem::size_of::<ViewerPanelBlock>());

/// Where IDA 0x5C291C is now (in the race block).
pub fn piece_addr() -> usize {
    crate::race::store::home(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(
            offset_of!(ViewerPanelBlock, bar_frame_rect),
            0x5C297C - START
        );
        assert_eq!(offset_of!(ViewerPanelBlock, mark_rect), 0x5C29DC - START);
        assert_eq!(
            offset_of!(ViewerPanelBlock, place_picture_rect),
            0x5C2AEC - START
        );
        assert_eq!(
            offset_of!(ViewerPanelBlock, texel_scale_small),
            0x5C2C68 - START
        );
        assert_eq!(
            offset_of!(ViewerPanelBlock, kept_patch_switches),
            0x5C2C98 - START
        );
        assert_eq!(
            offset_of!(ViewerPanelBlock, settings_text),
            0x5C2F84 - START
        );
        assert_eq!(
            offset_of!(ViewerPanelBlock, listener_turn),
            0x5C2FAC - START
        );
    }
}
