//! `menu::replay_viewer::frame`: one frame of the replay viewer (`screens::run_frame`'s function). At the
//! replay's end (`replay_at_end`) the screen is cleared (`gfx::clear_back_surface`); the scene
//! begins; unless at the end, the block's cameras are put into it (`drive::replay_frame_marks`)
//! and the frame is drawn (`race::show_replay_still`). When a picture is asked
//! (`snapshot_asked`), the request is cleared, the scene ends (`d3d::end_scene`), the picture is
//! saved (`race::viewer_snapshot`) and shown. Else, when stopped (`viewer_stopped` 1), the list's
//! texture (`viewer_list_texture`, `d3d::set_texture`), eight render states, each item, the bar's
//! quads (`drive::tyre_quad`: the frame, with blocks the bar, the two marks and the place shown
//! along it, the four buttons) and the panel (`menu::replay_viewer::panel`) are drawn; the scene
//! ends and the frame is shown (`gfx::show_frame`).
//!
//! The marks' and the place's positions are `mark * w / blocks`: a quotient past 32 bits is
//! refused, before the bar is drawn.

use crate::errors::{BadInput, Refused};
use crate::memory::session::PlainDword;
use crate::owners::Owners;
use crate::race::owner::Race;

/// Draws one viewer frame; returns whether the frame was lost.
pub(super) fn viewer_frame(o: Owners<'static>) -> Result<bool, Refused> {
    let (system, gfx, race) = (o.system, o.gfx, o.race);
    let replay = &race.replay;
    if race.shared.replay_file().replay_at_end.is_set() {
        crate::gfx::clear_back_surface::clear_back_surface(gfx, system)?;
    }
    // Beginning the scene cannot fail.
    crate::d3d::device::begin_layer(&o.gfx.d3d, o.scene.scratch.game_scratch());
    if !race.shared.replay_file().replay_at_end.is_set() {
        crate::drive::replay_frame_marks::replay_frame_marks(race)?;
        crate::race::show_replay_still::show_replay_still(race, o)?;
    }
    if replay.replay_view().snapshot_asked.get() == 1 {
        replay.replay_view().snapshot_asked.set(0);
        // Nor can ending it.
        crate::d3d::device::end_layer(&o.gfx.d3d);
        crate::race::viewer_snapshot::viewer_snapshot(replay, gfx, o.system)?;
    } else {
        let stopped_now = race.shared.replay_file().viewer_stopped.is_set();
        if stopped_now {
            stopped(race, o)?;
        }
        crate::d3d::device::end_layer(&o.gfx.d3d);
        // The highlighted button's name goes over the finished frame; a render has none.
        if stopped_now && !crate::video::active() {
            crate::menu::replay_viewer::tip::draw(o)?;
        }
    }
    // Not shown: the frame is lost.
    Ok(!crate::shell::frame::show_frame_now(o))
}

/// The list, the bar and the panel (stopped).
fn stopped(race: &Race, o: Owners<'static>) -> Result<(), Refused> {
    let (list_state, system) = (&o.menus.list, o.system);
    let replay = &race.replay;
    let shared = &race.shared;
    use rc2k_core::race::viewer_flow::{LIST_STATES, bar_places, mark_x, place_x};
    let part = o.scene.scratch.game_scratch();
    crate::d3d::set_texture::set_texture(part, shared.replay_file().viewer_list_texture.get());
    for (value, state) in LIST_STATES {
        crate::d3d::set_render_state::set_render_state(part, value, state);
    }
    let list = &replay.replay_view().viewer_list;
    for (index, item) in (0..).zip(&list.items) {
        crate::menu::replay_viewer::item::draw_viewer_item(
            race, list_state, item, index, system, o.scene,
        )?;
    }
    crate::menu::replay_viewer::render_row::draw_button(race, list_state, system, o.scene)?;
    // The dark strip and the view-name row span the screen; the time bar's middle stretches
    // between its two ends, which keep the 4:3 spacing from the time pieces at the screen's
    // edges; the marks and the place on it move with it but keep their shape.
    use rc2k_core::hud::anchor::Spread;
    let quad_spread = |uv: &[PlainDword], rect: &[PlainDword; 4], colour: u32, spread: Spread| {
        // The texture rectangle's first four dwords (the frame's picture holds five).
        let uv = uv
            .first_chunk::<4>()
            .ok_or(BadInput("a texture rectangle of fewer than four"))?;
        crate::drive::tyre_quad::tyre_quad(
            uv.each_ref().map(PlainDword::get),
            rect,
            colour,
            race.viewer_panel(),
            system,
            o.scene.scratch.game_scratch(),
            spread,
        )
    };
    let v = race.viewer_panel();
    let (from, to) = (
        v.bar_rect[0].get(),
        v.bar_rect[0].get().wrapping_add(v.bar_rect[2].get()),
    );
    let quad = |uv: &[PlainDword], rect: &[PlainDword; 4], colour: u32| {
        use rc2k_core::hud::anchor::Edge;
        let is = |r: &[PlainDword; 4]| std::ptr::eq(rect, r);
        let span = |from: u32, from_edge, to: u32, to_edge, keep_shape| Spread::Span {
            from,
            from_edge,
            to,
            to_edge,
            keep_shape,
        };
        let end = |r: &[PlainDword; 4]| r[0].get().wrapping_add(r[2].get());
        let spread = if is(&v.bar_rect) {
            span(from, Edge::Left, to, Edge::Right, false)
        } else if is(&v.mark_rect) || is(&v.place_rect) {
            span(from, Edge::Left, to, Edge::Right, true)
        } else if is(&v.button_rect_a) {
            // The view-name row: player 1's light at the left edge, player 2's in the middle,
            // each name's strip stretched from its light to the next piece.
            Spread::Left
        } else if is(&v.button_rect_c) {
            Spread::Centred
        } else if is(&v.button_rect_b) {
            span(
                v.button_rect_b[0].get(),
                Edge::Left,
                end(&v.button_rect_b),
                Edge::Centre,
                false,
            )
        } else if is(&v.button_rect_d) {
            span(
                v.button_rect_d[0].get(),
                Edge::Centre,
                end(&v.button_rect_d),
                Edge::Right,
                false,
            )
        } else {
            Spread::Fill
        };
        quad_spread(uv, rect, colour, spread)
    };
    quad(&v.frame_picture_rect, &v.bar_frame_rect, 0xFF_FFFF)?;
    let blocks = shared.replay_file().index.block_count.get();
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
            replay.replay_view().replay_block.get(),
            replay.replay_view().replay_frame.get(),
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
        crate::menu::replay_viewer::panel::draw_viewer_panel(
            crate::strings::append_string_by_id::StringOwners::of(o),
            race,
            o.gfx,
            o.strings,
            o.system,
            o.scene,
        )?;
    }
    Ok(())
}
