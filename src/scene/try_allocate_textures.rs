//! `scene::try_allocate_textures` (0x4224C4): asks for the texture memory every needed
//! texture takes, at the levels they are needed at.
//!
//! - Each kind's first slot and first texture (`kind_first_slots`,
//!   `kind_first_textures`) = -1.
//! - The memory asked for is added up for the log: `video_memory_asked` over the textures
//!   needed in video memory and `system_memory_asked` over those in system memory, by their
//!   levels' sizes; "VMEMREQ <n> SMEMREQ <n>" is logged.
//! - Then for each of the 0x400 textures: if it is needed in video memory, its level
//!   (held to 0 .. 7) is added to both sums again and one video texture is asked for
//!   from `gfx::allocate_textures` with the pixel kind (3 for 18 textures by their number,
//!   else 2) and the mip levels (`mip_setting` times the level's entry of `level_mips`;
//!   none for textures 0x80 .. 0xFF and from 0x140 on: `scene::no_mips_for`); what comes
//!   back is the texture's first slot. If it is needed in system memory: the same for a
//!   system texture with the mip levels as they are; what comes back is its first texture,
//!   and `texture_of` at that number = the texture.
//! - A refusal from `gfx::allocate_textures` ends it with "NO -" in the log and the
//!   carry; else "YES -".
//!
//! 46 call sites in `scene::build_texture_catalogue`. Refuses a level of 13 or more (past the
//! sizes' table).

use crate::errors::BadInput;
use crate::logfile::log_int::log_int;
use crate::race::owner_shared::RaceShared;
use crate::scene::owner::scene;
use crate::scene::texture_needs::TEXTURES;
use crate::scene::texture_places_store::cell_at;
use rc2k_core::Invalid;

// ---- game data ----

/// The memory asked for is kept in `video_memory_asked` and `system_memory_asked`.
/// Mip levels a level has: `OBJECT_TABLES.level_mips` (the sizes' table from its fifth entry).
pub use rc2k_core::scene::textures::Call;

// ---- Rust version ----

/// What a texture of `level` takes. The sizes' table goes on into the levels' mip figures and
/// the switch after them; the callers hold `level` below `LEVELS` (13).
pub fn level_size(tables: &crate::scene::object_tables_store::ObjectTables, level: usize) -> u32 {
    match level {
        0..4 => tables.level_sizes[level].get(),
        4..12 => tables.level_mips[level - 4].get(),
        _ => tables.other_levels.get(),
    }
}

/// The game's tables.
struct GameMemory<'a>(&'a RaceShared);

impl rc2k_core::scene::textures::TextureMemory for GameMemory<'_> {
    fn level(&self, video: bool, texture: u32) -> u32 {
        let objects = &scene().objects;
        let table = if video {
            &objects.tables.video_levels
        } else {
            &objects.tables.system_levels
        };
        table[texture as usize].get()
    }
    fn size(&self, level: u32) -> u32 {
        level_size(&scene().objects.tables, level as usize)
    }
    fn mips(&self) -> u32 {
        self.0.vars.mip_setting.get()
    }
    fn level_mips(&self, level: u32) -> u32 {
        scene().objects.tables.level_mips[level as usize].get()
    }
    fn forget_kinds(&mut self) {
        let textures = &scene().textures;
        for table in [
            &textures.kinds.kind_first_slots,
            &textures.kinds.kind_first_textures,
        ] {
            for cell in &table[..TEXTURES] {
                cell.set(u32::MAX);
            }
        }
    }
    fn set_asked(&mut self, asked: [u32; 2]) {
        let textures = &scene().textures;
        textures.kinds.video_memory_asked.set(asked[0]);
        textures.kinds.system_memory_asked.set(asked[1]);
    }
    fn set_first_slot(&mut self, texture: u32, slot: u32) {
        scene().textures.kinds.kind_first_slots[texture as usize].set(slot);
    }
    fn set_first_texture(&mut self, texture: u32, first: u32) -> Result<(), Invalid> {
        let textures = &scene().textures;
        cell_at(
            &textures.places,
            core::mem::offset_of!(
                crate::scene::texture_places_store::TexturePlaces,
                texture_of
            ),
            first,
        )
        .map_err(|e| Invalid(e.0))?
        .set(texture);
        textures.kinds.kind_first_textures[texture as usize].set(first);
        Ok(())
    }
}

/// Whether all the memory was had.
// Converted: `rc2k_core::scene::textures::try_allocate`
pub fn try_allocate_textures(
    shared: &RaceShared,
    call: &mut dyn FnMut(Call) -> Option<u32>,
) -> Result<bool, BadInput> {
    Ok(rc2k_core::scene::textures::try_allocate(
        &mut GameMemory(shared),
        &mut |c| call(c),
    )?)
}

/// With the game's log and the game's allocation.
pub fn try_allocate_textures_in_game(shared: &RaceShared) -> Result<bool, BadInput> {
    try_allocate_textures(shared, &mut |call| {
        match call {
            Call::Text(text, newline) => crate::logfile::log_write::log_nul(&text, newline, 1),
            Call::Number(number) => log_int(number as i32, false, 1),
            Call::Allocate(ask) => {
                // A failure of the allocation raises its fatal error; it never comes back
                // as a refusal.
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
