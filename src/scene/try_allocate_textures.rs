//! `scene::try_allocate_textures`: asks for the texture memory every needed texture takes, at
//! the levels they are needed at.
//!
//! - Each kind's first slot and first texture (`kind_first_slots`, `kind_first_textures`) = -1.
//! - The memory asked for is added up for the log: the sizes of the textures needed in video
//!   memory, and of those in system memory, by their levels; "VMEMREQ `<n>` SMEMREQ `<n>`" is
//!   logged.
//! - Then for each of the 0x400 textures: if it is needed in video memory, its level (held to
//!   0 ..= 7) is used and one video texture is asked for from
//!   `gfx::allocate_textures` with the pixel kind (3 for 18 textures by their number, else 2)
//!   and the mip levels (`mip_setting` times the level's entry of `level_mips`; none for
//!   textures 0x80 ..= 0xFF and from 0x140 on: `scene::no_mips_for`); what comes back is the
//!   texture's first slot. If it is needed in system memory: the same for a system texture with
//!   the mip levels as they are; what comes back is its first texture, and `texture_of` at that
//!   number = the texture.
//! - A refusal from `gfx::allocate_textures` ends it with "NO -" in the log and the carry; else
//!   "YES -".
//!
//! 46 call sites in `scene::build_texture_catalogue`. Refuses a level of 13 or more (past the
//! sizes' table).

use crate::errors::BadInput;
use crate::logfile::log_int::log_int;
use crate::race::owner_shared::RaceShared;
use crate::scene::owner::Scene;
use crate::scene::texture_needs::TEXTURES;
use rc2k_core::Invalid;

// ---- game data ----

/// Mip levels a level has: `OBJECT_TABLES.level_mips` (the sizes' table from its fifth entry).
pub use rc2k_core::scene::textures::Call;

/// What a texture of `level` takes. The sizes' table goes on into the levels' mip figures and
/// the switch after them; the callers hold `level` below `LEVELS` (13).
pub fn level_size(tables: &crate::scene::object_tables_store::TextureLevels, level: usize) -> u32 {
    match level {
        0..4 => tables.level_sizes[level],
        4..12 => tables.level_mips[level - 4],
        _ => tables.other_levels,
    }
}

/// The game's tables.
struct GameMemory<'a> {
    shared: &'a RaceShared,
    scene: &'a Scene,
}

impl rc2k_core::scene::textures::TextureMemory for GameMemory<'_> {
    fn level(&self, video: bool, texture: u32) -> u32 {
        let levels = self.scene.texture_levels();
        let table = if video {
            &levels.video_levels
        } else {
            &levels.system_levels
        };
        table[texture as usize]
    }
    fn size(&self, level: u32) -> u32 {
        level_size(&self.scene.texture_levels(), level as usize)
    }
    fn mips(&self) -> u32 {
        self.shared.vars.get().mip_setting.get()
    }
    fn level_mips(&self, level: u32) -> u32 {
        self.scene.texture_levels().level_mips[level as usize]
    }
    fn forget_kinds(&mut self) {
        let textures = &self.scene.textures;
        for cell in &textures.kinds.get().kind_first_slots[..TEXTURES] {
            cell.set(u32::MAX);
        }
        textures.first_textures_mut()[..TEXTURES].fill(u32::MAX);
    }
    fn set_first_slot(&mut self, texture: u32, slot: u32) {
        self.scene.textures.kinds.get().kind_first_slots[texture as usize].set(slot);
    }
    fn set_first_texture(&mut self, texture: u32, first: u32) -> Result<(), Invalid> {
        let textures = &self.scene.textures;
        *self
            .scene
            .texture_places_mut()
            .texture_of
            .get_mut(first as usize)
            .ok_or(Invalid(
                "a texture's first slot past the texture places' table",
            ))? = texture;
        textures.first_textures_mut()[texture as usize] = first;
        Ok(())
    }
}

/// Whether all the memory was had, with the game's log and the game's allocation.
// Logic: `rc2k_core::scene::textures::try_allocate`
pub fn try_allocate_textures(
    shared: &RaceShared,
    scene: &Scene,
    gfx: &crate::gfx::owner::Gfx,
    d3d: &crate::d3d::owner::D3d,
) -> Result<bool, BadInput> {
    Ok(rc2k_core::scene::textures::try_allocate(
        &mut GameMemory { shared, scene },
        &mut |call| {
            match call {
                Call::Text(text, newline) => crate::logfile::log_write::log_nul(&text, newline, 1),
                Call::Number(number) => log_int(number as i32, false, 1),
                Call::Allocate(ask) => {
                    // A failure of the allocation raises its fatal error; it never comes back
                    // as a refusal.
                    let (video, system) = crate::d3d::device::allocate_textures_for(
                        ask.video as u32,
                        !ask.video as u32,
                        ask.level,
                        ask.pixel_kind,
                        ask.mips,
                        gfx,
                        d3d,
                    );
                    return Some(if ask.video { video } else { system });
                }
            }
            Some(0)
        },
    )?)
}
