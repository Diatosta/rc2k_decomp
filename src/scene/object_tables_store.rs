//! The object slots' kinds and the texture level tables: the slots' kinds, the objects' kinds, the
//! object and point totals, the car's mark textures, the 3D switch, the level sizes and the three
//! 0x400-entry tables of texture tags and levels. The kind texture tables follow
//! (`scene::kind_texture_store`).

use crate::cells::ZeroCells;
use crate::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ObjectTables {
    /// Each object slot's kind (`scene::load_front_end_objects`,
    /// `scene::set_up_front_end_scene`, `race::objects`; cleared by `scene::clear_slot_kinds`).
    /// [64] is the kind kept by `race::objects` (`scene::draw_object_polygon`).
    pub slot_kinds: [PlainDword; 256],
    /// Each loaded object's kind (`scene::load_object`, `scene::objects`,
    /// `scene::list_object_face`, `physics::face_contact`, `drive::animate_lit_faces`).
    pub object_kinds: [PlainDword; 256],
    /// The faces loaded so far (`scene::add_object_shape`, `scene::load_object`).
    pub face_count: PlainDword,
    /// The points loaded so far (`scene::add_object_shape`, `scene::project_point`,
    /// `scene::place_object_points`).
    pub point_count: PlainDword,
    /// Where the shape's normals are in its file (`scene::add_object_shape`). Not used: nothing
    /// reads it, so the address is not kept here.
    pub normals_at: PlainDword,
    /// The textures of the car's marks and the menus' kind bytes. Unused: the drawing has its
    /// own in `scene::car_marks`.
    pub car_mark_textures: [PlainDword; 256],
    /// The car's letter number, a second copy, never read; unused.
    pub car_letter_b: PlainDword,
    /// Whether the Direct3D device is drawing: 0 until `race::draw_race_scene` first sets it to
    /// 1 (nothing sets it back); while 0 the texture, depth-clear and polygon functions skip
    /// the device. The switch the texture and polygon functions test
    /// (`scene::clear_depth_buffer`, `scene::submit_polygon`, `race::draw_race_scene`,
    /// `race::frame_views`).
    pub d3d_drawing_on: PlainDword,
    /// Seven dwords between the 3D switch and the level sizes; nothing reads or writes them (a
    /// gap).
    pub unknown_c14: [PlainDword; 7],
    /// What a texture of each level takes, in k (`scene::texture_needs`).
    pub level_sizes: [PlainDword; 4],
    /// The levels' mip figures (`scene::try_allocate_textures`).
    pub level_mips: [PlainDword; 8],
    /// Set when the other levels are in use (`scene::texture_levels`).
    pub other_levels: PlainDword,
    /// Each texture's tag (`scene::texture_needs`, `scene::texture_levels`).
    pub texture_tags: [PlainDword; 1024],
    /// The level each texture is needed at in the system memory (-1 for none; reset to 0xFF by
    /// `scene::reset_texture_tables`). [256] is the mud texture's halvings (`race::mud_texel`).
    pub system_levels: [PlainDword; 1024],
    /// The same for the video memory.
    pub video_levels: [PlainDword; 1024],
}

impl ObjectTables {
    /// The kind of `object`: the object numbers the game's code passes are not all bounded
    /// by the table, so a number past it reads on into the fields after it, in order, to the
    /// end of the record (a number past that ends the program).
    pub fn object_kind(&self, object: u32) -> u32 {
        if let Some(kind) = self.object_kinds.get(object as usize) {
            return kind.get();
        }
        crate::session::read_runs(
            &[
                &self.object_kinds,
                &self.face_count,
                &self.point_count,
                &self.normals_at,
                &self.car_mark_textures,
                &self.car_letter_b,
                &self.d3d_drawing_on,
                &self.unknown_c14,
                &self.level_sizes,
                &self.level_mips,
                &self.other_levels,
                &self.texture_tags,
                &self.system_levels,
                &self.video_levels,
            ],
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
    use super::*;

    /// Every object number from the table to the end of the record gives the entry of
    /// the fields after the table.
    #[test]
    fn object_kind_runs_on_through_the_fields_after() {
        use core::slice::from_ref;
        let t = Box::new(ObjectTables::new());
        let parts: [&[PlainDword]; 14] = [
            &t.object_kinds,
            from_ref(&t.face_count),
            from_ref(&t.point_count),
            from_ref(&t.normals_at),
            &t.car_mark_textures,
            from_ref(&t.car_letter_b),
            from_ref(&t.d3d_drawing_on),
            &t.unknown_c14,
            &t.level_sizes,
            &t.level_mips,
            from_ref(&t.other_levels),
            &t.texture_tags,
            &t.system_levels,
            &t.video_levels,
        ];
        let chain: Vec<&PlainDword> = parts.into_iter().flatten().collect();
        for (n, cell) in chain.iter().enumerate() {
            cell.set(0x6000 + n as u32);
        }
        for n in 0..chain.len() {
            assert_eq!(t.object_kind(n as u32), 0x6000 + n as u32, "object {n}");
        }
    }
}
