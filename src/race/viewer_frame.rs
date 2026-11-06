//! `race::viewer_frame` (0x41D5FB): one frame of the replay viewer (`screens::run_frame`'s
//! function): at the replay's end (`dword_5BEC90`) the screen cleared
//! (`gfx::clear_back_surface`); the scene begun (`d3d::begin_scene`; carry: lost); unless
//! at the end, the block's cameras into it (`drive::replay_frame_marks`) and the frame
//! drawn (`race::show_replay_still`). A picture asked (`dword_5C2540`): asked no more,
//! the scene ended (`d3d::end_scene`), the picture saved (`race::viewer_snapshot`), shown.
//! Else, stopped (`dword_5BEC98` 1): the list's texture (`dword_5BECB4`,
//! `d3d::set_texture`), eight render states, each item (`race::draw_viewer_item`, ecx its
//! number, `[edi+1]` of them), the bar's quads (`drive::tyre_quad`: the frame, with blocks
//! the bar, the two marks and the place shown along it, the four buttons) and the panel
//! (`race::draw_viewer_panel`); the scene ended and the frame shown (`gfx::show_frame`).
//!
//! In:  nothing
//! Out: carry set when the frame was lost (the scene's begin or end, the showing); every
//!      register kept (`pusha` / `popa`; `screens::run_frame` reads only carry)
//!
//! The marks' and the place's positions are `mark * w / blocks` (`mul` / `div`): a quotient
//! past 32 bits (the original's divide error) is refused, before the bar is drawn. Caller:
//! `screens::run_frame` (eax, from `race::replay_viewer`).

use crate::errors::{BadInput, Refused};
use crate::race::owner::race;
use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::viewer_screen_store::VIEWER_PANEL;
use crate::session::Dword;

/// The Rust version: whether the frame was lost (the carry the original leaves; it
/// keeps the other registers).
pub fn viewer_frame() -> Result<bool, Refused> {
    let replay = &race().replay;
    if REPLAY_FILE.replay_at_end.get() == 1 {
        crate::gfx::clear_back_surface::clear_back_surface()?;
    }
    // `d3d::begin_scene` cannot fail (its carry is clear).
    crate::platform_3d::begin_layer();
    if REPLAY_FILE.replay_at_end.get() != 1 {
        crate::drive::replay_frame_marks::replay_frame_marks()?;
        crate::race::show_replay_still::show_replay_still()?;
    }
    if replay.replay_view.snapshot_asked.get() == 1 {
        replay.replay_view.snapshot_asked.set(0);
        // `d3d::end_scene` cannot fail either.
        crate::platform_3d::end_layer();
        crate::race::viewer_snapshot::viewer_snapshot()?;
    } else {
        if REPLAY_FILE.viewer_stopped.get() == 1 {
            stopped()?;
        }
        crate::platform_3d::end_layer();
    }
    // `gfx::show_frame`'s carry says nothing was shown.
    Ok(!crate::platform::show_frame_now())
}

/// The list, the bar and the panel (stopped).
fn stopped() -> Result<(), Refused> {
    let replay = &race().replay;
    // Converted: `rc2k_core::race::viewer_flow::{LIST_STATES, bar_places, mark_x, place_x}`.
    use rc2k_core::race::viewer_flow::{LIST_STATES, bar_places, mark_x, place_x};
    crate::d3d::set_texture::set_texture(REPLAY_FILE.viewer_list_texture.get());
    for (value, state) in LIST_STATES {
        crate::d3d::set_render_state::set_render_state(value, state);
    }
    // The list's item count: the byte at +1 of its first dword.
    let items = (replay.replay_view.viewer_list[0].get() >> 8) as u8;
    // The original counts ecx up to the byte at +1 after each item, from 0: a count of 0
    // goes round 256 times. The items are the list's 16-byte entries from +0x18; one past
    // the 26 there is refused.
    for index in 0..if items == 0 { 256 } else { u32::from(items) } {
        let first = 6 + 4 * index as usize;
        let item = replay
            .replay_view
            .viewer_list
            .get(first..)
            .and_then(|rest| rest.first_chunk::<4>())
            .ok_or(BadInput("a viewer item past the 26"))?;
        crate::race::draw_viewer_item::draw_viewer_item(item, index)?;
    }
    let quad = |uv: &[Dword], rect: &[Dword; 4], colour: u32| {
        // The texture rectangle's first four dwords (the frame's picture holds five).
        let uv = uv
            .first_chunk::<4>()
            .ok_or(BadInput("a texture rectangle of fewer than four"))?;
        crate::drive::tyre_quad::tyre_quad_cells(uv, rect, colour)
    };
    let v = &VIEWER_PANEL;
    quad(&v.frame_picture_rect, &v.bar_frame_rect, 0xFF_FFFF)?;
    let blocks = REPLAY_FILE.index_block_count.get();
    if blocks != 0 {
        let (w, x) = (
            VIEWER_PANEL.bar_rect[2].get(),
            VIEWER_PANEL.bar_rect[0].get(),
        );
        let [first, second, here] = bar_places(
            (
                crate::race::viewer_buffers_store::VIEWER_BUFFERS
                    .mark_in
                    .get(),
                crate::race::viewer_buffers_store::VIEWER_BUFFERS
                    .mark_out
                    .get(),
            ),
            replay.replay_view.replay_block.get(),
            replay.replay_view.replay_frame.get(),
            w,
            blocks,
        )?;
        quad(&v.button_body_rect, &v.bar_rect, 0xFF00)?;
        VIEWER_PANEL.mark_rect[0].set(mark_x(first.0, x));
        quad(&v.mark_picture_rect, &v.mark_rect, 0xFF_FFFF)?;
        VIEWER_PANEL.mark_rect[0].set(mark_x(second.0, x));
        quad(&v.mark_picture_rect, &v.mark_rect, 0xFF_FFFF)?;
        VIEWER_PANEL.place_rect[0].set(place_x(here.0, x));
        quad(&v.place_picture_rect, &v.place_rect, 0xFF_FFFF)?;
        quad(&v.button_picture_rect, &v.button_rect_a, 0xFF_FFFF)?;
        quad(&v.button_picture_rect, &v.button_rect_c, 0xFF_FFFF)?;
        quad(&v.button_body_rect, &v.button_rect_b, 0xFF_FFFF)?;
        quad(&v.button_body_rect, &v.button_rect_d, 0xFF_FFFF)?;
        crate::race::draw_viewer_panel::draw_viewer_panel()?;
    }
    Ok(())
}
