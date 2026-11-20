//! `scene::begin_scene`: the 3D part of a frame begins.
//!
//! `d3d::begin_scene` (Direct3D's BeginScene and the layer's own states), then, whatever it says,
//! `scene::set_filters` and render states 0x15 = 4 (the texture's blending), 0x1C = 1 (fog on),
//! 0x13 = 5 and 0x14 = 6 (the source and destination blend), 0x1B = 1 (blending on).

use crate::scene::set_filters::set_filters;
use crate::scene::set_render_state::set_render_state;

/// The states after the filters: (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (1, 0x1C), (5, 0x13), (6, 0x14), (1, 0x1B)];

/// The 3D part of a frame begun. The layer's beginning cannot fail.
pub fn begin_scene(
    race: &crate::race::owner::Race,
    d3d: &crate::d3d::owner::D3d,
    part: &crate::scene::workers::PartScratch,
) {
    crate::d3d::device::begin_layer(d3d, part);
    set_filters(race, part);
    for (value, state) in STATES {
        set_render_state(part, value, state);
    }
}
