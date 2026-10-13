//! Which textures are needed, and how well: three tables of 0x400 dwords, one entry a
//! texture. Shared by the hooks that fill and count them.

use crate::framework::{game_addr, rva};
use crate::service::harness::Region;

/// Each texture's tag: which set of kinds it belongs to.
pub const TAGS_RVA: usize = rva(0x5CDAE0); // dword_5CDAE0
/// The level each is needed at in the two memories (-1 = not needed).
pub const SYSTEM_LEVELS_RVA: usize = rva(0x5CEAE0); // dword_5CEAE0
pub const VIDEO_LEVELS_RVA: usize = rva(0x5CFAE0); // dword_5CFAE0
pub const TEXTURES: usize = 0x400;
/// What a texture of each level takes, in k.
pub const LEVEL_SIZES_RVA: usize = rva(0x5CDAAC); // dword_5CDAAC
pub const LEVELS: u32 = 13;

pub fn levels(rva: usize) -> Region {
    (game_addr(rva) as u32, TEXTURES * 4)
}
