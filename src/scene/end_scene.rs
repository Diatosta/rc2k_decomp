//! `scene::end_scene`: the 3D part of a frame ends: a call of `d3d::end_scene` (Direct3D's
//! EndScene) and nothing more. Nine callers.

/// The 3D part of a frame ended: true when a scene was open.
pub fn end_scene(d3d: &crate::d3d::owner::D3d) -> bool {
    crate::platform::direct3d::end_layer(d3d)
}
