//! The race's HUD (`hud::draw_hud` and under it, entered from `hud::overlay_pass`): 2D polygons in
//! a virtual screen (`HudBlock.virtual_screen[0]` by `HudBlock.virtual_screen[1]`) scaled to the real one and gathered into a
//! batch (`HudBlock.vertices[0].x`, 0x400 vertices; `HudBlock.indices[0]`, 0x600 indices) drawn with one indexed
//! call per texture.

mod add_polygon;
mod add_quad;
mod add_scaled_sprite;
mod add_solid_quad;
mod add_sprite;
mod add_triangle;
mod batch;
pub(crate) mod box_ratio;
mod dials;
mod disc_vertex;
mod draw_disc;
mod draw_hud;
mod draw_icon;
mod draw_map_cars;
mod draw_marker;
mod draw_markers;
mod draw_needle;
mod draw_signs;
mod draw_stage_bar;
mod draw_stage_bars;
mod draw_view_strip;
mod draw_view_strips;
mod flush_batch;
mod icons;
mod layout_0;
mod layout_1;
mod layout_2;
pub(crate) mod layout_common;
mod layout_many;
pub(crate) mod overlay_pass;
pub(crate) mod overlay_states;
pub(crate) mod owner;
pub(crate) mod safe_inset;
mod set_batch_depth;
mod set_batch_texture;
mod store;
mod tenths;
mod turned_vertex;
