//! The race's HUD (`hud::draw_hud` and under it, entered from `hud::overlay_pass`): 2D polygons in
//! a virtual screen (`HudBlock.virtual_screen[0]` by `HudBlock.virtual_screen[1]`) scaled to the real one and gathered into a
//! batch (`HudBlock.vertices[0].x`, 0x400 vertices; `HudBlock.indices[0]`, 0x600 indices) drawn with one indexed
//! call per texture.

pub mod add_polygon;
pub mod add_quad;
pub mod add_scaled_sprite;
pub mod add_solid_quad;
pub mod add_sprite;
pub mod add_triangle;
pub mod batch;
pub mod box_ratio;
pub mod dials;
pub mod disc_vertex;
pub mod draw_disc;
pub mod draw_hud;
pub mod draw_icon;
pub mod draw_marker;
pub mod draw_markers;
pub mod draw_needle;
pub mod draw_signs;
pub mod draw_stage_bar;
pub mod draw_stage_bars;
pub mod draw_view_strip;
pub mod draw_view_strips;
pub mod flush_batch;
pub mod icons;
pub mod layout_0;
pub mod layout_1;
pub mod layout_2;
pub mod layout_common;
pub mod layout_many;
pub mod overlay_pass;
pub mod overlay_states;
pub mod owner;
pub mod part_scope;
pub mod set_batch_depth;
pub mod set_batch_texture;
pub mod store;
pub mod tenths;
pub mod turned_vertex;
