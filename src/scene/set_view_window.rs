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

use crate::math::x87;
use crate::math::x87_float as bh;
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
    vars: &ViewVars,
    scale_factor: &crate::memory::session::ConstFloat,
) {
    set_view_window_reaching(
        course,
        draw,
        screen,
        window,
        vars,
        scale_factor,
        course.stage_prefs.read(|p| p.grid_reach),
    );
}

/// `set_view_window` with the grid's reach `reach` in place of the stage's (the mirror's
/// window is made with a reach of 2).
pub fn set_view_window_reaching(
    course: crate::race::owner_course::CourseReads<'_>,
    draw: &SceneDraw,
    screen: &crate::memory::session::Screen,
    window: &ViewWindow,
    vars: &ViewVars,
    scale_factor: &crate::memory::session::ConstFloat,
    reach: u32,
) {
    let depth = reach.wrapping_mul(0x20);
    let [width, height]: [u32; 2] = window.borrow().view_size;
    let middle = [width >> 1, height >> 1];
    let offsets: [u32; 2] = window.borrow().view_offset;
    // top, bottom, left, right
    let edges = if course.number_vars.in_race() {
        [0, height, 0, width]
    } else {
        [
            offsets[1].wrapping_neg(),
            height.wrapping_sub(offsets[1]),
            offsets[0].wrapping_neg(),
            width.wrapping_sub(offsets[0]),
        ]
    };
    let zoom = window.borrow().view_zoom;
    let view_scale = bh::stf(
        bh::int(
            rc2k_core::scene::view_window::design_width(screen.width.get(), screen.height.get())
                as i32,
        ) * bh::ld(vars.view_scale)
            * bh::int(zoom as i32)
            * bh::ld(scale_factor.get()),
    );
    // Held to 90 ..= 100, compared by the signs of the differences as the game does.
    let mut nearer = 100u32.wrapping_sub(course.stage_prefs.read(|p| p.nearer));
    if (nearer.wrapping_sub(0x5A) as i32) < 0 {
        nearer = 0x5A;
    }
    if (nearer.wrapping_sub(0x64) as i32) >= 0 {
        nearer = 0x64;
    }
    let divisor = x87::int_times(nearer as i32, course.numbers.hundredth_a);
    let product = depth as u64 * 0x1_0000;
    let low = product as u32;
    let farthest = x87::int_over(low as i32, divisor);
    let farthest_scaled = x87::mul_div_div_to_int(
        farthest,
        draw.fade_a,
        draw.fade_b,
        course.numbers.k65536.get(),
    ) as u32;
    let clip_farthest = x87::mul_to_int(farthest, draw.clip_factor) as u32;
    let float = |value: u32| x87::from_int(value as i32);
    let mut window = window.borrow_mut();
    window.view_depth = [depth, depth];
    window.view_middle = middle;
    window.clip_edges = edges;
    window.view_scale = view_scale;
    window.farthest = farthest;
    window.one_over_farthest = x87::reciprocal(farthest);
    window.farthest_scaled = farthest_scaled;
    window.clip_farthest = clip_farthest;
    window.view_offset_float = offsets.map(float);
    window.clip_screen = edges.map(float);
    window.view_centre = middle.map(float);
}

/// `set_view_window` in plain floats. The x87 version above stays for the callers that have not
/// been converted (race, `choose_view_window`, `prepare_menu_scene`).
// Logic: `rc2k_core::scene::view_window::window_settings`
pub fn set_view_window_plain(
    course: crate::race::owner_course::CourseReads<'_>,
    draw: &SceneDraw,
    design: (u32, u32),
    window: &ViewWindow,
    vars: &ViewVars,
    scale_factor: &crate::memory::session::ConstFloat,
) {
    use rc2k_core::scene::view_window::{WindowInputs, window_settings};
    let (size, offsets, zoom) = {
        let window = window.borrow();
        (window.view_size, window.view_offset, window.view_zoom)
    };
    let i = WindowInputs {
        reach: course.stage_prefs.read(|p| p.grid_reach),
        size,
        offsets,
        in_race: course.number_vars.in_race(),
        screen_width: rc2k_core::scene::view_window::design_width(design.0, design.1),
        scale: (vars.view_scale, zoom, scale_factor.get()),
        nearer: course.stage_prefs.read(|p| p.nearer),
        nearer_factor: course.numbers.hundredth_a.get(),
        fade: [draw.fade_a, draw.fade_b],
        turn: course.numbers.k65536.get(),
        clip_factor: draw.clip_factor,
        one: course.numbers.one.get(),
    };
    let w = window_settings(&i);
    let mut window = window.borrow_mut();
    window.view_depth = [w.depth, w.depth];
    window.view_middle = w.middle;
    window.clip_edges = w.edges;
    window.view_scale = w.scale;
    window.farthest = w.farthest;
    window.one_over_farthest = w.reciprocal;
    window.farthest_scaled = w.fade_depth as u32;
    window.clip_farthest = w.clip_farthest as u32;
    window.view_offset_float = w.offset_floats;
    window.clip_screen = w.edge_floats;
    window.view_centre = w.middle_floats;
}
