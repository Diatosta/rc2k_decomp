//! The race's HUD (`sub_4B1CD0` and under it, entered from `sub_45A893`): 2D polygons in a
//! virtual screen (`flt_75C02C` by `flt_75C030`) scaled to the real one and gathered into a
//! batch (`unk_75C324`, 0x400 vertices; `unk_764324`, 0x600 indices) drawn with one indexed
//! call per texture.

pub mod add_polygon;
pub mod add_quad;
pub mod add_scaled_sprite;
pub mod add_solid_quad;
pub mod add_sprite;
pub mod add_triangle;
pub mod batch;
pub mod dials;
pub mod disc_vertex;
pub mod draw_disc;
pub mod draw_hud;
pub mod draw_icon;
pub mod draw_marker;
pub mod draw_markers_1;
pub mod draw_markers_2;
pub mod draw_needle;
pub mod draw_signs;
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
pub mod overlay_pass;
pub mod overlay_states;
pub mod set_batch_depth;
pub mod set_batch_texture;
pub mod store;
pub mod tenths;
pub mod turned_vertex;
