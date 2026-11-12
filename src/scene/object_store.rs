//! The scene objects' work (IDA 0x634FA0 .. 0x635118): the objects' loading
//! (memory used, the kind and slot being loaded, a shape's counts and firsts), the object
//! being drawn (its place, position, size, angles, faces) and the light levels. Names are
//! readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ObjectWork {
    /// (0x634FA0): the bytes of face memory used (`scene::objects`).
    pub face_cursor: Dword,
    /// (0x634FA4): "total pointmem" (`scene::load_front_end_objects`).
    pub point_memory_total: Dword,
    /// (0x634FA8): "total facemem".
    pub face_memory_total: Dword,
    /// (0x634FAC): eight dwords between the memory totals and the shape's counts; no code references them (a gap).
    pub unknown_c: [Dword; 8],
    /// (0x634FCC): the shape's counts (`scene::add_object_shape`): [0] faces, [1] points, [3] normals, [5] lowest, [6] highest.
    pub shape_counts: [Dword; 7],
    /// (0x634FE8): `g_loadingKind`: the kind of the object being loaded.
    pub loading_kind: Dword,
    /// (0x634FEC): `g_loadingSlot`: the slot being loaded.
    pub loading_slot: Dword,
    /// (0x634FF0): the car being loaded, its part, the car's first slot (`scene::load_object`; [1] read by `scene::add_object_shape`).
    pub loading_car: [Dword; 3],
    /// (0x634FFC): `g_objectPlace` (`scene::draw_described_object`, `scene::set_object_fade`); now each view's (`scene::object_place::ObjectPlace`); unused.
    pub object_place: Dword,
    /// (0x635000): one dword between the object's place and its position; no code references it (a gap).
    pub unknown_60: Dword,
    /// (0x635004): the object's x, y, depth (`scene::set_object_fade`, `physics::fill_race_slot`); the drawing's is each view's (`ctx.view.object.position`); this one is `physics::fill_race_slot`'s own.
    pub object_position: [Dword; 3],
    /// (0x635010): (`race::frame_draw`); now each view's (`ctx.view.object.face_depth`); unused.
    pub face_depth: Dword,
    /// (0x635014): cleared by `race::draw_view_scene`; now each view's (`ObjectPlace::view_cleared`); unused.
    pub view_cleared: [Dword; 2],
    /// (0x63501C): the matrix's place, x y z (`scene::matrix`); now each view's (`ctx.view.object.place`); read by none.
    pub place: [Dword; 3],
    /// (0x635028): (`race::car_draw`); now each view's (`ctx.view.object.car_switches`); unused.
    pub car_switches: Dword,
    /// (0x63502C): the place turned (`race::frame_draw`); now each view's (`ctx.view.object.turned_place`); unused.
    pub turned_place: [Dword; 3],
    /// (0x635038): the object's size (`scene::matrix`); each view's drawing uses `ctx.view.object.size`; this one is the physics' contact tests' own.
    pub size: Dword,
    /// (0x63503C): its angles, x y z: the physics' (`physics::object_turn_matrix`); the
    /// drawing's are the view's (`scene::object_place`).
    pub angles: [Dword; 3],
    /// (0x635048): (`race::car_parts`).
    pub part_values: [Dword; 3],
    /// (0x635054): the other light; now each view's (`scene::object_light`), this one
    /// not used.
    pub other_light: Dword,
    /// (0x635058): (`scene::draw_shown_object`); now each view's (`scene::object_place::ObjectPlace`); unused.
    pub only_object: Dword,
    /// (0x63505C): cleared by `scene::draw_described_object`; now each view's (`ObjectPlace::described_turns`); unused.
    pub described_cleared: [Dword; 3],
    /// (0x635068): (`scene::vertex_light`); now each view's (`scene::object_light`); unused.
    pub light_added: Dword,
    /// (0x63506C): two dwords between the light added and the part weights; no code references them (a gap).
    pub unknown_cc: [Dword; 2],
    /// (0x635074): (`race::car_parts`).
    pub part_weights: [Dword; 4],
    /// (0x635084): `g_lightLowest`; now each view's (`scene::object_light`); unused.
    pub light_lowest: Dword,
    /// (0x635088): `g_lightHighest`; now each view's (`scene::object_light`); unused.
    pub light_highest: Dword,
    /// (0x63508C): `g_lightLevel`; now each view's (`scene::object_light`); unused.
    pub light_level: Dword,
    /// (0x635090): `g_lightStrength`; now each view's (`ctx.view.object.light_strength`); read by none.
    pub light_strength: Dword,
    /// (0x635094): the light's turns, y then x (`scene::matrix`); now each view's (`scene::object_light`); unused.
    pub light_turns: [Dword; 2],
    /// (0x63509C): eight dwords after the light's turns; no code references them (a gap).
    pub unknown_fc: [Dword; 8],
    /// (0x6350BC): `g_objectFirstFace`; now each view's (`ctx.view.object.first_face`); unused.
    pub object_first_face: Dword,
    /// (0x6350C0): `g_objectFaceCount`; now each view's (`ctx.view.object.face_count`); unused.
    pub object_face_count: Dword,
    /// (0x6350C4): the shape's first point, face, normal (`scene::add_object_shape`).
    pub shape_firsts: [Dword; 3],
    /// (0x6350D0): 18 dwords closing the piece; the original touches only 0x6350E4 from undefined code, the port nothing (a gap).
    pub unknown_130: [Dword; 18],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl ObjectWork {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self {
            object_place: Dword::new(0xFFFFFFFF),
            ..ZeroCells::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scene::owner::scene;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(scene().object_work());
    }
}
