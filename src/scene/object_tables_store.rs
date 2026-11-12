//! The object slots' kinds and the texture level tables, 0x5CCE7C .. 0x5D0AE0: `g_slotKinds`,
//! `g_objectKinds`, the object and point totals, the car's mark textures, the 3D switch, the
//! level sizes and the three 0x400-entry tables of texture tags and levels. The kind texture
//! tables follow (`scene::kind_texture_store`).
//! Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::PlainDword;

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ObjectTables {
    /// +0x0 (0x5CCE7C): `g_slotKinds`: each object slot's kind (`scene::load_front_end_objects`, `scene::set_up_front_end_scene`, `race::objects`; cleared by `scene::clear_slot_kinds`). [64] is `dword_5CCF7C`, the kind kept by `race::objects` (`scene::draw_object_polygon`).
    pub slot_kinds: [PlainDword; 256],
    /// +0x400 (0x5CD27C): `g_objectKinds`: each loaded object's kind (`scene::load_object`, `scene::objects`, `scene::list_object_face`, `physics::face_contact`, `drive::animate_lit_faces`).
    pub object_kinds: [PlainDword; 256],
    /// +0x800 (0x5CD67C): `dword_5CD67C`: the faces loaded so far (`scene::add_object_shape`, `scene::load_object`).
    pub face_count: PlainDword,
    /// +0x804 (0x5CD680): `g_objectPointCount`: the points loaded so far (`scene::add_object_shape`, `scene::project_point`, `scene::place_object_points`).
    pub point_count: PlainDword,
    /// +0x808 (0x5CD684): `dword_5CD684`: where the shape's normals are in its file (`scene::add_object_shape`). Not used: nothing
    /// reads it, so the address is not kept here.
    pub normals_at: PlainDword,
    /// +0x80c (0x5CD688): `g_carMarkTextures`: the textures of the car's marks and the menus' kind bytes; now the drawing's own (`scene::car_marks`); unused.
    pub car_mark_textures: [PlainDword; 256],
    /// +0xc0c (0x5CDA88): `dword_5CDA88`: the car's letter number, a second copy, never read; unused.
    pub car_letter_b: PlainDword,
    /// +0xc10 (0x5CDA8C): `dword_5CDA8C`: whether the Direct3D device is drawing: 0 until `race::draw_race_scene` first sets it to 1 (nothing sets it back); while 0 the texture, depth-clear and polygon functions skip the device. The switch the texture and polygon functions test (`scene::clear_depth_buffer`, `scene::submit_polygon`, `race::draw_race_scene`, `race::frame_views`).
    pub d3d_drawing_on: PlainDword,
    /// +0xc14 (0x5CDA90): seven dwords between the 3D switch and the level sizes; nothing reads or writes them (a gap).
    pub unknown_c14: [PlainDword; 7],
    /// +0xc30 (0x5CDAAC): `g_textureLevelSizes`: what a texture of each level takes, in k (`scene::texture_needs`).
    pub level_sizes: [PlainDword; 4],
    /// +0xc40 (0x5CDABC): `dword_5CDABC`: the levels' mip figures (`scene::try_allocate_textures`).
    pub level_mips: [PlainDword; 8],
    /// +0xc60 (0x5CDADC): `g_otherTextureLevels`: set when the other levels are in use (`scene::texture_levels`).
    pub other_levels: PlainDword,
    /// +0xc64 (0x5CDAE0): `g_textureTags`: each texture's tag (`scene::texture_needs`, `scene::texture_levels`).
    pub texture_tags: [PlainDword; 1024],
    /// +0x1c64 (0x5CEAE0): `g_textureSystemLevels`: the level each texture is needed at in the system memory (-1 for none; reset to 0xFF by `scene::reset_texture_tables`). [256] is `g_mudTextureHalvings` (`race::mud_texel`).
    pub system_levels: [PlainDword; 1024],
    /// +0x2c64 (0x5CFAE0): `g_textureVideoLevels`: the same for the video memory.
    pub video_levels: [PlainDword; 1024],
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

impl ObjectTables {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self {
            d3d_drawing_on: PlainDword::new(1),
            unknown_c14: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0x100),
                PlainDword::new(0x80),
                PlainDword::new(0x40),
                PlainDword::new(0x20),
            ],
            level_sizes: [
                PlainDword::new(0x80),
                PlainDword::new(0x20),
                PlainDword::new(8),
                PlainDword::new(2),
            ],
            level_mips: [
                PlainDword::new(2),
                PlainDword::new(1),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
            ],
            ..ZeroCells::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scene::owner::scene;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            scene().objects.tables.get(),
        );
    }
}
