//! The race's HUD (`sub_4B1CD0` and under it, entered from `sub_45A893`): 2D polygons in a
//! virtual screen (`flt_75C02C` by `flt_75C030`) scaled to the real one and gathered into a
//! batch (`unk_75C324`, 0x400 vertices; `unk_764324`, 0x600 indices) drawn with one indexed
//! call per texture.

use crate::framework::GameHook;

pub mod add_polygon;
pub mod add_quad;
pub mod add_scaled_sprite;
pub mod add_solid_quad;
pub mod add_sprite;
pub mod add_triangle;
pub mod batch;
pub mod dials;
pub mod disc_0;
pub mod disc_1a;
pub mod disc_1b;
pub mod disc_2a;
pub mod disc_2b;
pub mod disc_vertex;
pub mod draw_disc;
pub mod draw_hud;
pub mod draw_icon;
pub mod draw_marker;
pub mod draw_markers_1;
pub mod draw_markers_2;
pub mod draw_needle;
pub mod draw_signs;
pub mod draw_signs_1a;
pub mod draw_signs_1b;
pub mod draw_signs_2a;
pub mod draw_signs_2b;
pub mod draw_stage_bar;
pub mod draw_stage_bars_1;
pub mod draw_stage_bars_2;
pub mod draw_view_strip;
pub mod draw_view_strips;
pub mod flush_batch;
pub mod icons_0;
pub mod icons_1;
pub mod icons_2;
pub mod layout_0;
pub mod layout_1;
pub mod layout_2;
pub mod needle_0;
pub mod needle_1a;
pub mod needle_1b;
pub mod needle_2a;
pub mod needle_2b;
pub mod overlay_pass;
pub mod overlay_states;
pub mod set_batch_depth;
pub mod set_batch_texture;
pub mod tenths;
pub mod turned_vertex;

pub static HOOKS: &[&dyn GameHook] = &[
    &add_polygon::HOOK,
    &add_quad::HOOK,
    &add_scaled_sprite::HOOK,
    &add_solid_quad::HOOK,
    &add_sprite::HOOK,
    &add_triangle::HOOK,
    &disc_0::HOOK,
    &disc_1a::HOOK,
    &disc_1b::HOOK,
    &disc_2a::HOOK,
    &disc_2b::HOOK,
    &disc_vertex::HOOK,
    &draw_disc::HOOK,
    &draw_hud::HOOK,
    &draw_icon::HOOK,
    &draw_marker::HOOK,
    &draw_markers_1::HOOK,
    &draw_markers_2::HOOK,
    &draw_needle::HOOK,
    &draw_signs::HOOK,
    &draw_signs_1a::HOOK,
    &draw_signs_1b::HOOK,
    &draw_signs_2a::HOOK,
    &draw_signs_2b::HOOK,
    &draw_stage_bar::HOOK,
    &draw_stage_bars_1::HOOK,
    &draw_stage_bars_2::HOOK,
    &draw_view_strip::HOOK,
    &draw_view_strips::HOOK,
    &flush_batch::HOOK,
    &icons_0::HOOK,
    &icons_1::HOOK,
    &icons_2::HOOK,
    &layout_0::HOOK,
    &layout_1::HOOK,
    &layout_2::HOOK,
    &needle_0::HOOK,
    &needle_1a::HOOK,
    &needle_1b::HOOK,
    &needle_2a::HOOK,
    &needle_2b::HOOK,
    &overlay_pass::HOOK,
    &overlay_states::HOOK,
    &set_batch_depth::HOOK,
    &set_batch_texture::HOOK,
    &tenths::HOOK,
    &turned_vertex::HOOK,
];
