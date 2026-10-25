//! `scene::try_allocate_textures` (0x4224C4): asks for the texture memory every needed
//! texture takes, at the levels they are needed at.
//!
//! In:  nothing
//! Out: carry clear = all of it was had; carry set = not. The original leaves the
//!      registers changed; its callers keep them all. The Rust version keeps them.
//!
//! - Each kind's first slot and first texture (`g_kindFirstSlots`,
//!   `g_kindFirstTextures`) = -1.
//! - The memory asked for is added up for the log: `dword_5D0AE0` over the textures
//!   needed in video memory and `dword_5D0AE4` over those in system memory, by their
//!   levels' sizes; "VMEMREQ <n> SMEMREQ <n>" is logged.
//! - Then for each of the 0x400 textures: if it is needed in video memory, its level
//!   (held to 0 .. 7) is added to both sums again and one video texture is asked for
//!   from `gfx::allocate_textures` (0x4051ED, Direct3D: not ported) with eax = 1,
//!   ebx = 0, cl = the level, edx = the pixel kind (3 for 18 textures by their number,
//!   else 2) and ebp = the mip levels (`dword_623898` times the level's entry of
//!   `dword_5CDABC`; none for textures 0x80 .. 0xFF and from 0x140 on:
//!   `scene::no_mips_for` 0x46512F, inlined); what comes back in eax is the texture's
//!   first slot. If it is needed in system memory: the same with eax = 0, ebx = 1 and
//!   the mip levels as they are; what comes back in ebx is its first texture, and
//!   `dword_5D2AE8` at that number = the texture.
//! - A refusal from `gfx::allocate_textures` ends it with "NO -" in the log and the
//!   carry; else "YES -".
//!
//! 46 call sites in `scene::build_texture_catalogue`. Refuses a level of 13 or more (past the
//! sizes' table).

use crate::errors::BadInput;
use crate::logfile::log_int::log_int;
use crate::logfile::log_write::log_write;
use crate::race::vars_store::RACE_VARS;
use crate::scene::kind_texture_store::KIND_TEXTURES;
use crate::scene::object_tables_store::OBJECT_TABLES;
use crate::scene::texture_needs::TEXTURES;
use crate::scene::texture_places_store::TEXTURE_PLACES;
use crate::scene::texture_places_store::cell_at;

// ---- game data ----

/// `dword_5D0AE0` video, `dword_5D0AE4` system: the memory asked for.
/// Mip levels a level has: `OBJECT_TABLES.level_mips` (the sizes' table from its fifth entry).
pub use rc2k_core::scene::textures::Call;

// ---- Rust version ----

/// The game's tables.
struct GameMemory;

impl rc2k_core::scene::textures::TextureMemory for GameMemory {
    fn level(&self, video: bool, texture: u32) -> u32 {
        let table = if video {
            &OBJECT_TABLES.video_levels
        } else {
            &OBJECT_TABLES.system_levels
        };
        cell_at(&table[0], texture).get()
    }
    fn size(&self, level: u32) -> u32 {
        cell_at(&OBJECT_TABLES.level_sizes[0], level).get()
    }
    fn mips(&self) -> u32 {
        RACE_VARS.mip_setting.bits()
    }
    fn level_mips(&self, level: u32) -> u32 {
        cell_at(&OBJECT_TABLES.level_mips[0], level).get()
    }
    fn forget_kinds(&mut self) {
        unsafe {
            std::ptr::write_bytes(
                KIND_TEXTURES.kind_first_slots[0].addr() as *mut u8,
                0xFF,
                TEXTURES * 4,
            );
            std::ptr::write_bytes(
                KIND_TEXTURES.kind_first_textures[0].addr() as *mut u8,
                0xFF,
                TEXTURES * 4,
            );
        }
    }
    fn set_asked(&mut self, asked: [u32; 2]) {
        KIND_TEXTURES.textures_asked.set(asked[0]);
        KIND_TEXTURES.textures_asked_other.set(asked[1]);
    }
    fn set_first_slot(&mut self, texture: u32, slot: u32) {
        KIND_TEXTURES.kind_first_slots[texture as usize].set(slot)
    }
    fn set_first_texture(&mut self, texture: u32, first: u32) {
        cell_at(&TEXTURE_PLACES.texture_of[0], first).set(texture);
        KIND_TEXTURES.kind_first_textures[texture as usize].set(first);
    }
}

/// Whether all the memory was had.
// Converted: `rc2k_core::scene::textures::try_allocate`
pub unsafe fn try_allocate_textures(
    call: &mut dyn FnMut(Call) -> Option<u32>,
) -> Result<bool, BadInput> {
    Ok(rc2k_core::scene::textures::try_allocate(
        &mut GameMemory,
        &mut |c| call(c),
    )?)
}

/// With the game's log and the game's allocation.
pub unsafe fn try_allocate_textures_in_game() -> Result<bool, BadInput> {
    unsafe {
        try_allocate_textures(&mut |call| {
            match call {
                Call::Text(text, newline) => log_write(text.as_ptr().cast(), newline, 1),
                Call::Number(number) => log_int(number as i32, false, 1),
                Call::Allocate(ask) => {
                    // The allocation reads no esi (the texture) and never comes back
                    // with its carry set: a failure raises its fatal error.
                    let (video, system) = crate::platform_3d::allocate_textures_for(
                        ask.video as u32,
                        !ask.video as u32,
                        ask.level,
                        ask.pixel_kind,
                        ask.mips,
                    );
                    return Some(if ask.video { video } else { system });
                }
            }
            Some(0)
        })
    }
}
