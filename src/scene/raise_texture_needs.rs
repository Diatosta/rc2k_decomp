//! `scene::raise_texture_needs` (0x423F97): the textures of one tag are needed at a
//! level at least.
//!
//! In:  eax = the level, ebx = the tag
//! Out: ecx = 0x400, edx = 1 if any entry changed, else 0; zero set if any changed,
//!      carry set if none; everything else preserved
//!
//! For every texture whose tag (`dword_5CDAE0`) is ebx: its level in each of the two tables
//! (`dword_5CEAE0`, `dword_5CFAE0`) becomes eax if eax is above it, unsigned (so -1, "not
//! needed", is never raised). 57 callers.

use crate::scene::object_tables_store::OBJECT_TABLES;

// ---- Rust version ----

/// Whether any entry changed.
pub fn raise_texture_needs(level: u32, tag: u32) -> bool {
    // Converted: `rc2k_core::scene::textures::Needs::raise`.
    crate::scene::textures::with_cells(
        [
            &OBJECT_TABLES.texture_tags,
            &OBJECT_TABLES.system_levels,
            &OBJECT_TABLES.video_levels,
        ],
        |[tags, system, video]| {
            rc2k_core::scene::textures::Needs {
                tags,
                system,
                video,
            }
            .raise(level, tag)
        },
    )
}
