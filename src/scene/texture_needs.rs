//! Which textures are needed, and how well: three tables of 0x400 dwords, one entry a texture.
//! Shared by the functions that fill and count them.

/// The tables are `OBJECT_TABLES.texture_tags` (each texture's tag: which set of kinds it
/// belongs to), `system_levels` and `video_levels` (the level each is needed at in the two
/// memories, -1 = not needed) and `level_sizes` (what a texture of each level takes, in k).
pub const TEXTURES: usize = 0x400;
