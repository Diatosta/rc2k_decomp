//! `scene::set_view_window` (0x44E47B): the view's window, scale and depths from its
//! size and the scene's settings.
//!
//! In:  nothing
//! Out: eax = the low half and edx = the high half of `dword_63DC54` times 0x10000,
//!      ebx = 0x10000; everything else preserved
//!
//! From the window's width and height (`dword_62408C`, `dword_624090`), its offsets
//! (`dword_623FB0`, `dword_623FB4`) and `dword_60A87C` (how far is seen, a reading):
//! - `dword_63DC58` = `dword_63DC54` = `dword_60A87C` times 0x20;
//! - the middle (`dword_624094`, `dword_624098`) = half of each, unsigned;
//! - the clip rectangle as whole numbers `dword_6241D4` (top), `D8` (bottom), `DC`
//!   (left), `E0` (right): 0, 0, width, height with `g_inRace` at 1, else each less its
//!   offset;
//! - the scale `flt_624088` = the screen's width times `dword_606294` times
//!   `dword_624080` (a whole number) times `flt_623DCC`;
//! - `flt_624224` = (100 less `dword_60AC80`, held to 90 .. 100 by the differences'
//!   signs) times `flt_5C5CFC`; the farthest depth `flt_6241C8` = `dword_63DC54` times
//!   0x10000 (its low half, signed) over that, `flt_6241CC` = 1 over it,
//!   `dword_6241D0` = it times `flt_624228` over `flt_624220` over 65536, and
//!   `g_clipFarthest` = it times `flt_62422C`, both rounded;
//! - as floats: the offsets (`flt_6241F4`, `F8`), the clip rectangle
//!   (`g_clipScreenLeft` ..), the middle (`flt_62409C`, `flt_6240A0`), and the width
//!   and height less `flt_5C5D38` (`flt_6D18E8`, `EC`).
//!
//! `dword_62400C` is left holding the low half. Twelve callers.

use crate::math::x87;
use crate::race::numbers_store::NUMBERS;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::sound_spots_store::RACE_SOUND_SPOTS;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::view_vars_store::VIEW_VARS;
use crate::scene::draw_store::DRAW;
use crate::session::SCREEN;

// ---- Rust version ----

pub fn set_view_window() {
    let depth = STAGE_PREFS.grid_reach.bits().wrapping_mul(0x20);
    crate::session::write_bits(&RACE_VIEW.view_depth, [depth, depth]);
    let [width, height]: [u32; 2] = crate::session::read_bits(&DRAW.view_size);
    let middle = [width >> 1, height >> 1];
    crate::session::write_bits(&DRAW.view_middle, middle);
    let offsets: [u32; 2] = crate::session::read_bits(&DRAW.view_offset);
    // top, bottom, left, right
    let edges = if NUMBERS.in_race.bits() == 1 {
        [0, height, 0, width]
    } else {
        [
            offsets[1].wrapping_neg(),
            height.wrapping_sub(offsets[1]),
            offsets[0].wrapping_neg(),
            width.wrapping_sub(offsets[0]),
        ]
    };
    crate::session::write_bits(&DRAW.clip_edges, edges);
    DRAW.view_scale.set_bits(x87::int_float_int_float(
        SCREEN.width.get() as i32,
        VIEW_VARS.view_scale.bits(),
        DRAW.view_zoom.bits() as i32,
        RACE_VARS.scale_factor.bits(),
    ));
    // `cmp; jns` and `cmp; js`: by the differences' signs.
    let mut nearer = 100u32.wrapping_sub(STAGE_PREFS.nearer.bits());
    if (nearer.wrapping_sub(0x5A) as i32) < 0 {
        nearer = 0x5A;
    }
    if (nearer.wrapping_sub(0x64) as i32) >= 0 {
        nearer = 0x64;
    }
    let divisor = x87::int_times(nearer as i32, NUMBERS.hundredth_a.bits());
    DRAW.depth_divisor.set_bits(divisor);
    let product = depth as u64 * 0x1_0000;
    let low = product as u32;
    DRAW.scratch.set_bits(low);
    let farthest = x87::int_over(low as i32, divisor);
    DRAW.farthest.set_bits(farthest);
    DRAW.one_over_farthest.set_bits(x87::reciprocal(farthest));
    DRAW.farthest_scaled.set(x87::mul_div_div_to_int(
        farthest,
        DRAW.fade_a.bits(),
        DRAW.fade_b.bits(),
        NUMBERS.k65536.bits(),
    ) as u32);
    DRAW.clip_farthest
        .set(x87::mul_to_int(farthest, DRAW.clip_factor.bits()) as u32);
    let float = |value: u32| x87::from_int(value as i32);
    crate::session::write_bits(&DRAW.view_offset_float, offsets.map(float));
    crate::session::write_bits(&DRAW.clip_screen, edges.map(float));
    crate::session::write_bits(&DRAW.view_centre, middle.map(float));
    let one: u32 = NUMBERS.one.bits();
    crate::session::write_bits(
        &RACE_SOUND_SPOTS.less_one,
        [width, height].map(|size| x87::int_less(size as i32, one)),
    );
}

/// The converted `set_view_window` (plain floats). The x87 version above
/// stays for its Rust callers in modules not converted (race, `choose_view_window`,
/// `prepare_menu_scene`).
// Converted: `rc2k_core::scene::view_window::window_settings`
pub fn set_view_window_plain() {
    use rc2k_core::scene::view_window::{WindowInputs, window_settings};
    let i = WindowInputs {
        reach: STAGE_PREFS.grid_reach.bits(),
        size: crate::session::read_bits(&DRAW.view_size),
        offsets: crate::session::read_bits(&DRAW.view_offset),
        in_race: NUMBERS.in_race.bits() == 1,
        screen_width: SCREEN.width.get(),
        scale: (
            VIEW_VARS.view_scale.get(),
            DRAW.view_zoom.bits(),
            RACE_VARS.scale_factor.get(),
        ),
        nearer: STAGE_PREFS.nearer.bits(),
        nearer_factor: NUMBERS.hundredth_a.get(),
        fade: [DRAW.fade_a.get(), DRAW.fade_b.get()],
        turn: NUMBERS.k65536.get(),
        clip_factor: DRAW.clip_factor.get(),
        one: NUMBERS.one.get(),
    };
    let w = window_settings(&i);
    crate::session::write_bits(&RACE_VIEW.view_depth, [w.depth, w.depth]);
    crate::session::write_bits(&DRAW.view_middle, w.middle);
    crate::session::write_bits(&DRAW.clip_edges, w.edges);
    DRAW.view_scale.set_bits(w.scale.to_bits());
    DRAW.depth_divisor.set_bits(w.divisor.to_bits());
    let low = w.product as u32;
    DRAW.scratch.set_bits(low);
    DRAW.farthest.set(w.farthest);
    DRAW.one_over_farthest.set(w.reciprocal);
    DRAW.farthest_scaled.set(w.fade_depth as u32);
    DRAW.clip_farthest.set_bits(w.clip_farthest as u32);
    let set = |cells: &[crate::session::Float], v: &[f32]| {
        for (cell, &x) in cells.iter().zip(v) {
            cell.set(x);
        }
    };
    set(&DRAW.view_offset_float, &w.offset_floats);
    set(&DRAW.clip_screen, &w.edge_floats);
    set(&DRAW.view_centre, &w.middle_floats);
    set(&RACE_SOUND_SPOTS.less_one, &w.less_one);
}
