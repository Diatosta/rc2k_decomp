//! `scene::set_fog_colour`: the fog's colour: kept as the fade start and given to Direct3D
//! (render state 0x22 = the colour with its top byte set, through `d3d::set_render_state`).
//! Callers: `race::set_up_fog` and ten more.

// Logic: `rc2k_core::scene::render_states::fog_colour`
pub fn set_fog_colour(
    colour: u32,
    poly: &crate::scene::polygon_work::PolygonWork,
    part: &crate::scene::workers::PartScratch,
) {
    poly.fade_from.set(colour);
    let s = rc2k_core::scene::render_states::fog_colour(colour);
    crate::d3d::set_render_state::set_render_state(part, s.value, s.state);
}
