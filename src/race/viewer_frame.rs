//! `race::viewer_frame` (0x41D5FB): one frame of the replay viewer (`screens::run_frame`'s
//! function): at the replay's end (`replay_at_end`) the screen cleared (`gfx::clear_back_surface`);
//! the scene begun; unless at the end, the block's cameras into it (`drive::replay_frame_marks`)
//! and the frame drawn (`race::show_replay_still`). A picture asked (`snapshot_asked`): asked no
//! more, the scene ended (`d3d::end_scene`), the picture saved (`race::viewer_snapshot`), shown.
//! Else, stopped (`viewer_stopped` 1): the list's texture (`viewer_list_texture`, `d3d::set_texture`), eight
//! render states, each item, the bar's quads (`drive::tyre_quad`: the frame, with blocks the bar,
//! the two marks and the place shown along it, the four buttons) and the panel
//! (`race::draw_viewer_panel`); the scene ended and the frame shown (`gfx::show_frame`).
//!
//! The marks' and the place's positions are `mark * w / blocks` (`mul` / `div`): a quotient
//! past 32 bits (the original's divide error) is refused, before the bar is drawn. Caller:
//! `screens::run_frame` (called from `race::replay_viewer`).

use crate::errors::{BadInput, Refused};
use crate::owners::Owners;
use crate::race::owner::Race;
use crate::session::Dword;

/// Whether the frame was lost.
pub fn viewer_frame(o: Owners<'static>) -> Result<bool, Refused> {
    let (system, gfx, race) = (o.system, o.gfx, o.race);
    let replay = &race.replay;
    if race.shared.replay_file.replay_at_end.is_set() {
        crate::gfx::clear_back_surface::clear_back_surface(gfx, system)?;
    }
    // `d3d::begin_scene` cannot fail (its carry is clear).
    crate::platform_3d::begin_layer();
    if !race.shared.replay_file.replay_at_end.is_set() {
        crate::drive::replay_frame_marks::replay_frame_marks(race)?;
        crate::race::show_replay_still::show_replay_still(race, o)?;
    }
    if replay.replay_view.snapshot_asked.get() == 1 {
        replay.replay_view.snapshot_asked.set(0);
        // `d3d::end_scene` cannot fail either.
        crate::platform_3d::end_layer();
        crate::race::viewer_snapshot::viewer_snapshot(replay, gfx)?;
    } else {
        if race.shared.replay_file.viewer_stopped.is_set() {
            stopped(race, o)?;
        }
        crate::platform_3d::end_layer();
    }
    // `gfx::show_frame`'s carry says nothing was shown.
    Ok(!crate::platform::show_frame_now())
}

/// The list, the bar and the panel (stopped).
fn stopped(race: &Race, o: Owners<'static>) -> Result<(), Refused> {
    let (install, system) = (o.install, o.system);
    let replay = &race.replay;
    let shared = &race.shared;
    // Converted: `rc2k_core::race::viewer_flow::{LIST_STATES, bar_places, mark_x, place_x}`.
    use rc2k_core::race::viewer_flow::{LIST_STATES, bar_places, mark_x, place_x};
    crate::d3d::set_texture::set_texture(shared.replay_file.viewer_list_texture.get());
    for (value, state) in LIST_STATES {
        crate::d3d::set_render_state::set_render_state(value, state);
    }
    // The list's item count: the byte at +1 of its first dword.
    let items = (replay.replay_view.viewer_list[0].get() >> 8) as u8;
    // The original counts up to the byte at +1 after each item, from 0: a count of 0
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
        crate::race::draw_viewer_item::draw_viewer_item(race, install, item, index, system)?;
    }
    let quad = |uv: &[Dword], rect: &[Dword; 4], colour: u32| {
        // The texture rectangle's first four dwords (the frame's picture holds five).
        let uv = uv
            .first_chunk::<4>()
            .ok_or(BadInput("a texture rectangle of fewer than four"))?;
        crate::drive::tyre_quad::tyre_quad_cells(
            uv,
            rect,
            colour,
            race.viewer_panel(),
            system,
            crate::workers::game_scratch().vertices,
        )
    };
    let v = race.viewer_panel();
    quad(&v.frame_picture_rect, &v.bar_frame_rect, 0xFF_FFFF)?;
    let blocks = shared.replay_file.index_block_count.get();
    if blocks != 0 {
        let (w, x) = (
            race.viewer_panel().bar_rect[2].get(),
            race.viewer_panel().bar_rect[0].get(),
        );
        let [first, second, here] = bar_places(
            (
                race.viewer_buffers().mark_in.get(),
                race.viewer_buffers().mark_out.get(),
            ),
            replay.replay_view.replay_block.get(),
            replay.replay_view.replay_frame.get(),
            w,
            blocks,
        )?;
        quad(&v.button_body_rect, &v.bar_rect, 0xFF00)?;
        race.viewer_panel().mark_rect[0].set(mark_x(first.0, x));
        quad(&v.mark_picture_rect, &v.mark_rect, 0xFF_FFFF)?;
        race.viewer_panel().mark_rect[0].set(mark_x(second.0, x));
        quad(&v.mark_picture_rect, &v.mark_rect, 0xFF_FFFF)?;
        race.viewer_panel().place_rect[0].set(place_x(here.0, x));
        quad(&v.place_picture_rect, &v.place_rect, 0xFF_FFFF)?;
        quad(&v.button_picture_rect, &v.button_rect_a, 0xFF_FFFF)?;
        quad(&v.button_picture_rect, &v.button_rect_c, 0xFF_FFFF)?;
        quad(&v.button_body_rect, &v.button_rect_b, 0xFF_FFFF)?;
        quad(&v.button_body_rect, &v.button_rect_d, 0xFF_FFFF)?;
        crate::race::draw_viewer_panel::draw_viewer_panel(race, o.gfx, o.strings, o.system)?;
    }
    Ok(())
}
