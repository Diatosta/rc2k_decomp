//! The object slots' kinds and the texture level tables, 0x5CCE7C .. 0x5D0AE0: `g_slotKinds`,
//! `g_objectKinds`, the object and point totals, the car's mark textures, the 3D switch, the
//! level sizes and the three 0x400-entry tables of texture tags and levels. The kind texture
//! tables follow (`scene::kind_texture_store`).
//! Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ObjectTables {
    /// +0x0 (0x5CCE7C): `g_slotKinds`: each object slot's kind (`scene::load_front_end_objects`, `scene::set_up_front_end_scene`, `race::objects`; cleared by `scene::clear_slot_kinds`). [64] is `dword_5CCF7C`, the kind kept by `race::objects` (`scene::draw_object_polygon`).
    pub slot_kinds: [Dword; 256],
    /// +0x400 (0x5CD27C): `g_objectKinds`: each loaded object's kind (`scene::load_object`, `scene::objects`, `scene::list_object_face`, `physics::face_contact`, `drive::animate_lit_faces`).
    pub object_kinds: [Dword; 256],
    /// +0x800 (0x5CD67C): `dword_5CD67C`: the faces loaded so far (`scene::add_object_shape`, `scene::load_object`).
    pub face_count: Dword,
    /// +0x804 (0x5CD680): `g_objectPointCount`: the points loaded so far (`scene::add_object_shape`, `scene::project_point`, `scene::place_object_points`).
    pub point_count: Dword,
    /// +0x808 (0x5CD684): `dword_5CD684`: where the shape's normals are in its file (`scene::add_object_shape`). Not used: nothing
    /// reads it, so the address is not kept here.
    pub normals_at: Dword,
    /// +0x80c (0x5CD688): `g_carMarkTextures`: the textures of the car's marks (number, digits, plates) in the first 0x40 (`race::car_draw`; [1] .. [9] are `dword_5CD68C` .. `dword_5CD6AC`), and the 0x100 kind bytes `scene::draw_shown_object` sets (`scene::list_object_face`).
    pub car_mark_textures: [Dword; 256],
    /// +0xc0c (0x5CDA88): `dword_5CDA88`: the car's letter number, a second copy (`race::car_number_textures`).
    pub car_letter_b: Dword,
    /// +0xc10 (0x5CDA8C): `dword_5CDA8C`: the 3D switch the texture and polygon functions test (`scene::clear_depth_buffer`, `scene::submit_polygon`, `race::draw_race_scene`, `race::frame_views`).
    pub has_3d: Dword,
    /// +0xc14 (0x5CDA90): not named.
    pub unknown_c14: [Dword; 7],
    /// +0xc30 (0x5CDAAC): `g_textureLevelSizes`: what a texture of each level takes, in k (`scene::texture_needs`).
    pub level_sizes: [Dword; 4],
    /// +0xc40 (0x5CDABC): `dword_5CDABC`: the levels' mip figures (`scene::try_allocate_textures`).
    pub level_mips: [Dword; 8],
    /// +0xc60 (0x5CDADC): `g_otherTextureLevels`: set when the other levels are in use (`scene::texture_levels`).
    pub other_levels: Dword,
    /// +0xc64 (0x5CDAE0): `g_textureTags`: each texture's tag (`scene::texture_needs`, `scene::texture_levels`).
    pub texture_tags: [Dword; 1024],
    /// +0x1c64 (0x5CEAE0): `g_textureSystemLevels`: the level each texture is needed at in the system memory (-1 for none; reset to 0xFF by `scene::reset_texture_tables`). [256] is `g_mudTextureHalvings` (`race::mud_texel`).
    pub system_levels: [Dword; 1024],
    /// +0x2c64 (0x5CFAE0): `g_textureVideoLevels`: the same for the video memory.
    pub video_levels: [Dword; 1024],
}

impl ObjectTables {
    /// `g_objectKinds[object]`, read by address: the object numbers the game's code passes
    /// are not all bounded by the table.
    pub fn object_kind(&self, object: u32) -> u32 {
        crate::session::read_on_in(
            self,
            &self.object_kinds,
            object as usize,
            "an object's kind",
        )
        .unwrap_or_else(|e| crate::fatal::raise::end("an object's kind", e))
    }
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static OBJECT_TABLES: ObjectTables =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x5CCE7C, the first byte of the piece.
pub const START: usize = 0x5CCE7C;
/// IDA 0x5D0AE0, the end of the piece.
pub const END: usize = 0x5D0AE0;

const _: () = assert!(END - START == core::mem::size_of::<ObjectTables>());

/// Where IDA 0x5CCE7C is now (its own static).
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&OBJECT_TABLES))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&OBJECT_TABLES, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(ObjectTables, slot_kinds), 0x5CCE7C - START);
        assert_eq!(offset_of!(ObjectTables, has_3d), 0x5CDA8C - START);
        assert_eq!(offset_of!(ObjectTables, video_levels), 0x5CFAE0 - START);
    }
}
