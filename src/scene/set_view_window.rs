//! `scene::set_view_window`: the view's window, scale and depths from its
//! size and the scene's settings.
//!
//! From the window's `view_size` and `view_offset`, the stage's `grid_reach` (how far the
//! grid reaches; the view's depth is it times 0x20):
//! - `view_depth` (both entries) = `grid_reach` times 0x20;
//! - `view_middle` = half of each size, unsigned;
//! - `clip_edges` (top, bottom, left, right) as whole numbers: 0, height, 0, width in a
//!   race, else each less its offset;
//! - `view_scale` = the screen's width times the view variables' `view_scale` times
//!   `view_zoom` (a whole number) times the race's `scale_factor`;
//! - `depth_divisor` = (100 less the stage's `nearer`, held to 90 .. 100 by the
//!   differences' signs) times `hundredth_a`; the farthest depth `farthest` = the view
//!   depth times 0x10000 (its low half, signed) over that, `one_over_farthest` = 1 over it,
//!   `farthest_scaled` = it times `fade_a` over `fade_b` over 65536, and `clip_farthest` =
//!   it times `clip_factor`, both rounded;
//! - as floats: the offsets (`view_offset_float`), the clip rectangle (`clip_screen`) and
//!   the middle (`view_centre`).
//!
//! Twelve callers.

use crate::math::x87;
use crate::math::x87_float as bh;
use crate::memory::session::{CellKind, FloatLike};
use crate::race::view_vars_store::ViewVars;
use crate::scene::draw_store::SceneDraw;
use crate::scene::view_window::ViewWindow;

/// `vars`: the view variables the window's scale is read from (`view_scale`: a view's own,
/// `ViewScratch::vars`, or the shared ones); `draw`: the scene's draw variables (the fade and
/// clip factors), a drawing part's from its reads; `screen`: the screen's figures (the part's
/// from its reads likewise).
pub fn set_view_window(
    course: crate::race::owner_course::CourseReads<'_>,
    draw: &SceneDraw,
    screen: &crate::memory::session::Screen,
    window: &ViewWindow,
    vars: &ViewVars<impl CellKind>,
    scale_factor: &crate::memory::session::PlainFloat,
) {
    set_view_window_reaching(
        course,
        draw,
        screen,
        window,
        vars,
        scale_factor,
        course.stage_prefs.grid_reach.get(),
    );
}

/// `set_view_window` with the grid's reach `reach` in place of the stage's (the mirror's
/// window is made with a reach of 2).
pub fn set_view_window_reaching(
    course: crate::race::owner_course::CourseReads<'_>,
    draw: &SceneDraw,
    screen: &crate::memory::session::Screen,
    window: &ViewWindow,
    vars: &ViewVars<impl CellKind>,
    scale_factor: &crate::memory::session::PlainFloat,
    reach: u32,
) {
    let depth = reach.wrapping_mul(0x20);
    crate::memory::session::write_bits(&window.view_depth, [depth, depth]);
    let [width, height]: [u32; 2] = crate::memory::session::read_bits(&window.view_size);
    let middle = [width >> 1, height >> 1];
    crate::memory::session::write_bits(&window.view_middle, middle);
    let offsets: [u32; 2] = crate::memory::session::read_bits(&window.view_offset);
    // top, bottom, left, right
    let edges = if course.numbers.in_race.is_set() {
        [0, height, 0, width]
    } else {
        [
            offsets[1].wrapping_neg(),
            height.wrapping_sub(offsets[1]),
            offsets[0].wrapping_neg(),
            width.wrapping_sub(offsets[0]),
        ]
    };
    crate::memory::session::write_bits(&window.clip_edges, edges);
    window.view_scale.set(bh::stf(
        bh::int(screen.width.get() as i32)
            * bh::ld(vars.view_scale.get())
            * bh::int(window.view_zoom.get() as i32)
            * bh::ld(scale_factor.get()),
    ));
    // Held to 90 ..= 100, compared by the signs of the differences as the game does.
    let mut nearer = 100u32.wrapping_sub(course.stage_prefs.nearer.get());
    if (nearer.wrapping_sub(0x5A) as i32) < 0 {
        nearer = 0x5A;
    }
    if (nearer.wrapping_sub(0x64) as i32) >= 0 {
        nearer = 0x64;
    }
    let divisor = x87::int_times(nearer as i32, &course.numbers.hundredth_a);
    window.depth_divisor.set(divisor);
    let product = depth as u64 * 0x1_0000;
    let low = product as u32;
    let farthest = x87::int_over(low as i32, divisor);
    window.farthest.set(farthest);
    window.one_over_farthest.set(x87::reciprocal(farthest));
    window.farthest_scaled.set(x87::mul_div_div_to_int(
        farthest,
        draw.fade_a.get(),
        draw.fade_b.get(),
        course.numbers.k65536.get(),
    ) as u32);
    window
        .clip_farthest
        .set(x87::mul_to_int(farthest, &draw.clip_factor) as u32);
    let float = |value: u32| x87::from_int(value as i32);
    crate::memory::session::write_bits(
        &window.view_offset_float,
        offsets.map(float).map(f32::to_bits),
    );
    crate::memory::session::write_bits(&window.clip_screen, edges.map(float).map(f32::to_bits));
    crate::memory::session::write_bits(&window.view_centre, middle.map(float).map(f32::to_bits));
}

/// `set_view_window` in plain floats. The x87 version above stays for the callers that have not
/// been converted (race, `choose_view_window`, `prepare_menu_scene`).
// Logic: `rc2k_core::scene::view_window::window_settings`
pub fn set_view_window_plain(
    course: crate::race::owner_course::CourseReads<'_>,
    draw: &SceneDraw,
    screen: &crate::memory::session::Screen,
    window: &ViewWindow,
    vars: &ViewVars<impl CellKind>,
    scale_factor: &crate::memory::session::PlainFloat,
) {
    use rc2k_core::scene::view_window::{WindowInputs, window_settings};
    let i = WindowInputs {
        reach: course.stage_prefs.grid_reach.get(),
        size: crate::memory::session::read_bits(&window.view_size),
        offsets: crate::memory::session::read_bits(&window.view_offset),
        in_race: course.numbers.in_race.is_set(),
        screen_width: screen.width.get(),
        scale: (
            vars.view_scale.get(),
            window.view_zoom.get(),
            scale_factor.get(),
        ),
        nearer: course.stage_prefs.nearer.get(),
        nearer_factor: course.numbers.hundredth_a.get(),
        fade: [draw.fade_a.get(), draw.fade_b.get()],
        turn: course.numbers.k65536.get(),
        clip_factor: draw.clip_factor.get(),
        one: course.numbers.one.get(),
    };
    let w = window_settings(&i);
    crate::memory::session::write_bits(&window.view_depth, [w.depth, w.depth]);
    crate::memory::session::write_bits(&window.view_middle, w.middle);
    crate::memory::session::write_bits(&window.clip_edges, w.edges);
    window.view_scale.set(w.scale);
    window.depth_divisor.set(w.divisor);
    window.farthest.set(w.farthest);
    window.one_over_farthest.set(w.reciprocal);
    window.farthest_scaled.set(w.fade_depth as u32);
    window.clip_farthest.set(w.clip_farthest as u32);
    let set = |cells: &[crate::memory::session::PlainFloat], v: &[f32]| {
        for (cell, &x) in cells.iter().zip(v) {
            cell.set(x);
        }
    };
    set(&window.view_offset_float, &w.offset_floats);
    set(&window.clip_screen, &w.edge_floats);
    set(&window.view_centre, &w.middle_floats);
}
