//! Which textures are needed, and how well: three tables of 0x400 dwords, one entry a texture.
//! Shared by the functions that fill and count them.

use crate::framework::rva;

/// Each texture's tag: which set of kinds it belongs to.
pub const TAGS_RVA: usize = rva(0x5CDAE0); // dword_5CDAE0
/// The level each is needed at in the two memories (-1 = not needed).
pub const SYSTEM_LEVELS_RVA: usize = rva(0x5CEAE0); // dword_5CEAE0
pub const VIDEO_LEVELS_RVA: usize = rva(0x5CFAE0); // dword_5CFAE0
pub const TEXTURES: usize = 0x400;
/// What a texture of each level takes, in k.
pub const LEVEL_SIZES_RVA: usize = rva(0x5CDAAC); // dword_5CDAAC
