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

use crate::errors::Refused;
use crate::physics::flow::{a, g, p};
use crate::race::viewer::{
    AT_END, BLOCK, BLOCKS, FRAME, MARK_IN, MARK_OUT, MENU, SNAPSHOT, STOPPED, gb,
};

const LIST_TEXTURE: usize = 0x5BECB4; // dword_5BECB4
const BAR_X: usize = 0x5C29CC; // dword_5C29CC: the bar's rectangle (x, .., w at +8)
const BAR_W: usize = 0x5C29D4; // dword_5C29D4
const MARK_RECT: usize = 0x5C29DC; // dword_5C29DC
const PLACE_RECT: usize = 0x5C29EC; // dword_5C29EC

/// The Rust version: whether the frame was lost (the carry the original leaves; it
/// keeps the other registers).
pub fn viewer_frame() -> Result<bool, Refused> {
    if g(AT_END) == 1 {
        crate::gfx::clear_back_surface::clear_back_surface()?;
    }
    // `d3d::begin_scene` cannot fail (its carry is clear).
    crate::platform_3d::begin_layer();
    if g(AT_END) != 1 {
        crate::drive::replay_frame_marks::replay_frame_marks();
        crate::race::show_replay_still::show_replay_still()?;
    }
    if g(SNAPSHOT) == 1 {
        p(SNAPSHOT, 0);
        // `d3d::end_scene` cannot fail either.
        crate::platform_3d::end_layer();
        crate::race::viewer_snapshot::viewer_snapshot()?;
    } else {
        if g(STOPPED) == 1 {
            stopped()?;
        }
        crate::platform_3d::end_layer();
    }
    // `gfx::show_frame`'s carry says nothing was shown.
    Ok(!crate::platform::show_frame_now())
}

/// The list, the bar and the panel (stopped).
fn stopped() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{LIST_STATES, bar_places, mark_x, place_x}`.
    use rc2k_core::race::viewer_flow::{LIST_STATES, bar_places, mark_x, place_x};
    crate::d3d::set_texture::set_texture(g(LIST_TEXTURE));
    for (value, state) in LIST_STATES {
        crate::d3d::set_render_state::set_render_state(value, state);
    }
    let menu = a(MENU);
    let items = gb(MENU + 1);
    let mut item = menu + 0x18;
    // The original counts ecx up to the byte at +1 after each item, from 0: a count of 0
    // goes round 256 times.
    for index in 0..if items == 0 { 256 } else { u32::from(items) } {
        crate::race::draw_viewer_item::draw_viewer_item(item, index)?;
        item += 0x10;
    }
    let quad = |uv: usize, rect: usize, colour: u32| unsafe {
        crate::drive::tyre_quad::tyre_quad(a(uv), a(rect), colour)
    };
    quad(0x5C2C4C, 0x5C297C, 0xFF_FFFF)?;
    let blocks = g(BLOCKS);
    if blocks != 0 {
        let (w, x) = (g(BAR_W), g(BAR_X));
        let [first, second, here] =
            bar_places((g(MARK_IN), g(MARK_OUT)), g(BLOCK), g(FRAME), w, blocks)?;
        quad(0x5C2C3C, 0x5C29CC, 0xFF00)?;
        p(MARK_RECT, mark_x(first.0, x));
        quad(0x5C2ACC, MARK_RECT, 0xFF_FFFF)?;
        p(MARK_RECT, mark_x(second.0, x));
        quad(0x5C2ACC, MARK_RECT, 0xFF_FFFF)?;
        p(PLACE_RECT, place_x(here.0, x));
        quad(0x5C2AEC, PLACE_RECT, 0xFF_FFFF)?;
        quad(0x5C2C2C, 0x5C298C, 0xFF_FFFF)?;
        quad(0x5C2C2C, 0x5C29AC, 0xFF_FFFF)?;
        quad(0x5C2C3C, 0x5C299C, 0xFF_FFFF)?;
        quad(0x5C2C3C, 0x5C29BC, 0xFF_FFFF)?;
        crate::race::draw_viewer_panel::draw_viewer_panel()?;
    }
    Ok(())
}
