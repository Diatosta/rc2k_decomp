//! `scene::clear_fog` (0x44FC19): no fog.
//!
//! Out: eax = 0x0, ecx = 0, edi = the end of the last table
//!
//! `dword_636610` = 0 and the 0x1000 bytes of `g_fogTable` = 0. (The original clears the table
//! first; the result is the same.) One caller.

// ---- game data ----

// ---- Rust version ----

pub fn clear_fog(scene: &crate::scene::owner::Scene) {
    let lights = &scene.lights;
    lights.state.fog_next.set_bits(0u32);
    for cell in &lights.state.fog_table {
        cell.set(0);
    }
}
