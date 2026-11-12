//! `scene::end_scene` (0x44E952): the 3D part of a frame ends.
//!
//! Out: as `d3d::end_scene` (0x41AB8A: Direct3D's EndScene, platform) leaves them
//!
//! A call of `d3d::end_scene` and nothing more. Nine callers. The registers above are the
//! original's; a call by address goes through `routes.rs`, which maps them.

/// The 3D part of a frame ended: true when a scene was open (the original then leaves
/// nothing a caller reads).
pub fn end_scene(d3d: &crate::d3d::owner::D3d) -> bool {
    crate::platform_3d::end_layer(d3d)
}
