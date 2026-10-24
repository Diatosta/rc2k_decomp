//! `d3d::set_texture` (0x41ACF5): makes a texture the one polygons are drawn with.
//!
//! In:  eax = the texture's number, or 0xFFFFFC43 for none
//! Out: ecx = 0; eax, edx as Direct3D leaves them
//! Nothing if it is the current one (`g_currentTexture`, 0x5AB93C); else that is set
//! and the device told: render state 1 with the texture's handle from the texture
//! table (0x78 bytes a texture at `dword_5ABC84`) and a light state, or render state 1
//! with 0 for none. `sub_41ACF0`, five bytes before it, loads eax with "none" and runs
//! into it. Four callers.
//! Platform: the original always runs, here it is only noted. In the standalone
//! program, and for a recording that swallows, the number is kept and nothing more.

use crate::framework::{game_write, rva};

pub const CURRENT_RVA: usize = rva(0x5AB93C); // g_currentTexture

/// The texture polygons are drawn with from now on (the standalone keeps its number).
/// Its leftover ecx (0) is read by none of the original's callers (register liveness,
/// 2026-10-23).
pub fn set_texture(texture: u32) {
    unsafe { game_write(CURRENT_RVA, texture) };
}
