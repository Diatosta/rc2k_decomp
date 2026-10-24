//! `scene::end_scene` (0x44E952): the 3D part of a frame ends.
//!
//! In:  nothing
//! Out: as `d3d::end_scene` (0x41AB8A: Direct3D's EndScene, platform) leaves them
//!
//! A call of `d3d::end_scene` and nothing more. Nine callers.
//! Hook removed 2026-10-22; plain Rust since 2026-10-23 (phase 3): the registers above
//! are the original's; a call by address goes through `routes.rs`, which maps them.

/// The 3D part of a frame ended: true when a scene was open (the original then leaves
/// eax 0 and the zero flag set; the carry always clear).
pub fn end_scene() -> bool {
    crate::platform_3d::end_layer()
}
