//! `scene::clear_depth_buffer` (0x44E967): clears the depth buffer over the whole
//! screen.
//!
//! In:  nothing
//! Out: nothing changed without 3D; else eax, ecx, edx as Direct3D leaves them and
//!      ebx = the screen's height
//!
//! Only with `dword_5CDA8C` not 0: the rectangle `dword_624138` .. = 0, 0, the
//! screen's width and height, and `d3d::clear_viewport` (0x44E9D9: the viewport's
//! Clear with that one rectangle and flag 2, the depth buffer; Direct3D, not ported).
//! Three callers.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose call notes the rectangle.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{game_read, game_write, rva};

// ---- game data ----

const HAS_3D_RVA: usize = rva(0x5CDA8C); // dword_5CDA8C
const RECTANGLE_RVA: usize = rva(0x624138); // dword_624138 .. dword_624144
const WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const HEIGHT_RVA: usize = rva(0x4F4950); // g_screenHeight

// ---- Rust version ----

/// The depth buffer cleared over the whole screen (`d3d::clear_viewport`), unless there
/// is no 3D. What the original leaves (eax 0, ebx the height) its three callers overwrite
/// before reading (each next calls `scene::assign_kind_textures`, which sets both).
pub fn clear_depth_buffer() {
    unsafe {
        if game_read::<u32>(HAS_3D_RVA) == 0 {
            return;
        }
        let (width, height): (u32, u32) = (game_read(WIDTH_RVA), game_read(HEIGHT_RVA));
        game_write(RECTANGLE_RVA, [0, 0, width, height]);
        crate::platform_3d::clear_viewport();
    }
}
