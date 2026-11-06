//! `scene::set_up_textures` (0x44E7A5): the texture memory for everything that is
//! needed, and one texture more for the game's own use.
//!
//! In:  nothing
//! Out: carry clear; eax and ebx as `gfx::allocate_textures` leaves them
//!
//! `scene::build_texture_catalogue`; `dword_624234` = 0; then one system texture of size code 3
//! and pixel kind 4 without mip levels from `gfx::allocate_textures` (0x4051ED, Direct3D: not
//! ported; eax = 0, ebx = 1, ecx = 3, edx = 4, ebp = 0), whose number (ebx) goes to
//! `dword_624234`. If that one is refused: fatal error 0x80600943. Two callers. The registers
//! above are the original's; a call by address goes through `routes.rs`, which maps them.

use crate::errors::Refused;
use crate::scene::build_texture_catalogue::build_texture_catalogue_in_game;

// ---- game data ----

// ---- Rust version ----

/// The catalogue built, then one system texture of the scene's own (sizes 3, pixel kind
/// 4, no mips; `gfx::allocate_textures`) kept in `dword_624234`. The allocation cannot
/// come back refused (a failure raises its own fatal error), so the original's fatal
/// error 0x80600943 ("cant alloc sys video texture") is never reached. The original
/// leaves the carry clear and eax, ebx as the allocation leaves them; no caller reads
/// those registers (register liveness, 2026-10-23: `race::textures_alloc` and
/// `race::loadlevel_init` after it), the carry the register-threaded callers set.
pub fn set_up_textures() -> Result<(), Refused> {
    build_texture_catalogue_in_game()?;
    crate::scene::owner::scene()
        .drawing
        .scene
        .mirror_picture
        .set_bits(0u32);
    let (_, system) = crate::platform_3d::allocate_textures_for(0, 1, 3, 4, 0);
    crate::scene::owner::scene()
        .drawing
        .scene
        .mirror_picture
        .set_bits(system);
    Ok(())
}
