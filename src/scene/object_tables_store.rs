//! The object slots' kinds and the texture level tables. `ObjectKinds` (the loaded objects' kinds,
//! the points loaded so far and the 3D switch) is what the drawing parts read, a plain record lent as a copy;
//! `TextureLevels` (the slots' kinds, the faces loaded, the level sizes and the three 0x400-entry
//! tables of texture tags and levels) is plain fields only the game's thread reaches
//! (`scene::owner::Scene::texture_levels`). The kind texture tables follow
//! (`scene::kind_texture_store`).

use crate::memory::session::{DrawPlain, PlainRead};

/// What the drawing parts read of the objects' tables: a plain record (`session::DrawPlain`)
/// written on the game's thread, read there and, as a copy made when a drawing scope opens, by the
/// parts through their reads (`scene::workers::DrawReads`).
#[derive(Clone)]
pub struct ObjectKinds {
    /// Each loaded object's kind (`scene::load_object`, `scene::objects`,
    /// `scene::list_object_face`, `physics::face_contact`, `drive::animate_lit_faces`).
    pub object_kinds: [u32; 256],
    /// The points loaded so far (`scene::add_object_shape`, `scene::project_point`,
    /// `scene::place_object_points`).
    pub point_count: u32,
    /// Whether the Direct3D device is drawing: 0 until `race::draw_race_scene` first sets it to
    /// 1 (nothing sets it back); while 0 the texture, depth-clear and polygon functions skip
    /// the device (`scene::clear_depth_buffer`, `scene::submit_polygon`, `race::draw_race_scene`,
    /// `race::frame_views`).
    pub d3d_drawing_on: u32,
}

impl ObjectKinds {
    /// The kind of `object` (an object number past the table ends the program).
    pub fn object_kind(&self, object: u32) -> u32 {
        match self.object_kinds.get(object as usize) {
            Some(&kind) => kind,
            None => crate::memory::host_hooks::end(
                "an object's kind",
                crate::errors::BadInput("an object number past the kinds' table"),
            ),
        }
    }

    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self {
            object_kinds: [0; 256],
            point_count: 0,
            d3d_drawing_on: 1,
        }
    }
}

/// The tables as one reader finds them: the game's thread's own (read through the store when used),
/// or a drawing part's copy.
#[derive(Clone, Copy)]
pub enum ObjectKindsRef<'a> {
    Store(&'a DrawPlain<ObjectKinds>),
    Copy(&'a ObjectKinds),
}

impl ObjectKindsRef<'_> {
    /// The kind of `object` (an object number past the table ends the program).
    pub fn object_kind(self, object: u32) -> u32 {
        match self {
            Self::Store(store) => store.borrow().object_kind(object),
            Self::Copy(tables) => tables.object_kind(object),
        }
    }

    /// Every object's kind.
    pub fn object_kinds(self) -> [u32; 256] {
        match self {
            Self::Store(store) => store.borrow().object_kinds,
            Self::Copy(tables) => tables.object_kinds,
        }
    }

    /// The points loaded so far.
    pub fn point_count(self) -> u32 {
        match self {
            Self::Store(store) => store.borrow().point_count,
            Self::Copy(tables) => tables.point_count,
        }
    }

    /// Whether the device is drawing (not 0).
    pub fn d3d_drawing_on(self) -> u32 {
        match self {
            Self::Store(store) => store.borrow().d3d_drawing_on,
            Self::Copy(tables) => tables.d3d_drawing_on,
        }
    }
}

impl<'a> From<&'a PlainRead<ObjectKinds>> for ObjectKindsRef<'a> {
    fn from(copy: &'a PlainRead<ObjectKinds>) -> Self {
        Self::Copy(copy)
    }
}

/// The slots' kinds and the texture level tables: plain fields, reached by the game's thread alone.
#[derive(Clone)]
pub struct TextureLevels {
    /// Each object slot's kind (`scene::load_front_end_objects`,
    /// `scene::set_up_front_end_scene`, `race::objects`; cleared by `scene::clear_slot_kinds`).
    /// [64] is the kind kept by `race::objects` (`scene::draw_object_polygon`).
    pub slot_kinds: [u32; 256],
    /// The faces loaded so far (`scene::add_object_shape`, `scene::load_object`).
    pub face_count: u32,
    /// What a texture of each level takes, in k (`scene::texture_needs`).
    pub level_sizes: [u32; 4],
    /// The levels' mip figures (`scene::try_allocate_textures`).
    pub level_mips: [u32; 8],
    /// Set when the other levels are in use (`scene::texture_levels`).
    pub other_levels: u32,
    /// Each texture's tag (`scene::texture_needs`, `scene::texture_levels`).
    pub texture_tags: [u32; 1024],
    /// The level each texture is needed at in the system memory (-1 for none; reset to 0xFF by
    /// `scene::reset_texture_tables`). [256] is the mud texture's halvings (`race::mud_texel`).
    pub system_levels: [u32; 1024],
    /// The same for the video memory.
    pub video_levels: [u32; 1024],
}

impl TextureLevels {
    /// The tables with their first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self {
            slot_kinds: [0; 256],
            face_count: 0,
            level_sizes: [0x80, 0x20, 8, 2],
            level_mips: [2, 1, 0, 0, 0, 0, 0, 0],
            other_levels: 0,
            texture_tags: [0; 1024],
            system_levels: [0; 1024],
            video_levels: [0; 1024],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tables start as the game has them.
    #[test]
    fn the_tables_start_as_the_game_has_them() {
        let levels = TextureLevels::new();
        assert_eq!(levels.level_sizes, [0x80, 0x20, 8, 2]);
        assert_eq!(levels.level_mips, [2, 1, 0, 0, 0, 0, 0, 0]);
        let kinds = ObjectKinds::new();
        assert_eq!(kinds.d3d_drawing_on, 1);
        assert_eq!(kinds.object_kind(255), 0);
    }
}
