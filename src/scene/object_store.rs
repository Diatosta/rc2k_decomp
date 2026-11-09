//! The scene objects' work (IDA 0x634FA0 .. 0x635118; phase 7, typed): the objects' loading
//! (memory used, the kind and slot being loaded, a shape's counts and firsts), the object
//! being drawn (its place, position, size, angles, faces) and the light levels. `#[repr(C)]`
//! with the image's layout, over the race block's bytes (`OBJECT_WORK`), as `CAR`. Names
//! are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ObjectWork {
    /// +0x0 (0x634FA0): the bytes of face memory used (`scene::objects`).
    pub face_cursor: Dword,
    /// +0x4 (0x634FA4): "total pointmem" (`scene::load_front_end_objects`).
    pub point_memory_total: Dword,
    /// +0x8 (0x634FA8): "total facemem".
    pub face_memory_total: Dword,
    /// +0xc (0x634FAC): not named.
    pub unknown_c: [Dword; 8],
    /// +0x2c (0x634FCC): the shape's counts (`scene::add_object_shape`): [0] faces, [1] points, [3] normals, [5] lowest, [6] highest.
    pub shape_counts: [Dword; 7],
    /// +0x48 (0x634FE8): `g_loadingKind`: the kind of the object being loaded.
    pub loading_kind: Dword,
    /// +0x4c (0x634FEC): `g_loadingSlot`: the slot being loaded.
    pub loading_slot: Dword,
    /// +0x50 (0x634FF0): the car being loaded, its part, the car's first slot (`scene::load_object`; [1] read by `scene::add_object_shape`).
    pub loading_car: [Dword; 3],
    /// +0x5c (0x634FFC): `g_objectPlace` (`scene::draw_described_object`, `scene::set_object_fade`).
    pub object_place: Dword,
    /// +0x60 (0x635000): not named.
    pub unknown_60: Dword,
    /// +0x64 (0x635004): the object's x, y, depth (`scene::set_object_fade`, `physics::fill_race_slot`); the drawing's is each view's (`ctx.view.object.position`); this one is `physics::fill_race_slot`'s own.
    pub object_position: [Dword; 3],
    /// +0x70 (0x635010): (`race::frame_draw`); now each view's (`ctx.view.object.face_depth`); unused.
    pub face_depth: Dword,
    /// +0x74 (0x635014): cleared by `race::draw_view_scene`.
    pub view_cleared: [Dword; 2],
    /// +0x7c (0x63501C): the matrix's place, x y z (`scene::matrix`); now each view's (`ctx.view.object.place`); read by none.
    pub place: [Dword; 3],
    /// +0x88 (0x635028): (`race::car_draw`); now each view's (`ctx.view.object.car_switches`); unused.
    pub car_switches: Dword,
    /// +0x8c (0x63502C): the place turned (`race::frame_draw`); now each view's (`ctx.view.object.turned_place`); unused.
    pub turned_place: [Dword; 3],
    /// +0x98 (0x635038): the object's size (`scene::matrix`); each view's drawing uses `ctx.view.object.size`; this one is the physics' contact tests' own.
    pub size: Dword,
    /// +0x9c (0x63503C): its angles, x y z (`scene::matrix`, `physics::object_turn_matrix`).
    pub angles: [Dword; 3],
    /// +0xa8 (0x635048): (`race::car_parts`).
    pub part_values: [Dword; 3],
    /// +0xb4 (0x635054): (`scene::draw_object_face`).
    pub other_light: Dword,
    /// +0xb8 (0x635058): (`scene::draw_shown_object`).
    pub only_object: Dword,
    /// +0xbc (0x63505C): cleared by `scene::draw_described_object`.
    pub described_cleared: [Dword; 3],
    /// +0xc8 (0x635068): (`scene::vertex_light`); now each view's (`scene::object_light`); unused.
    pub light_added: Dword,
    /// +0xcc (0x63506C): not named.
    pub unknown_cc: [Dword; 2],
    /// +0xd4 (0x635074): (`race::car_parts`).
    pub part_weights: [Dword; 4],
    /// +0xe4 (0x635084): `g_lightLowest`; now each view's (`scene::object_light`); unused.
    pub light_lowest: Dword,
    /// +0xe8 (0x635088): `g_lightHighest`; now each view's (`scene::object_light`); unused.
    pub light_highest: Dword,
    /// +0xec (0x63508C): `g_lightLevel`; now each view's (`scene::object_light`); unused.
    pub light_level: Dword,
    /// +0xf0 (0x635090): `g_lightStrength`; now each view's (`ctx.view.object.light_strength`); read by none.
    pub light_strength: Dword,
    /// +0xf4 (0x635094): the light's turns, y then x (`scene::matrix`); now each view's (`scene::object_light`); unused.
    pub light_turns: [Dword; 2],
    /// +0xfc (0x63509C): not named.
    pub unknown_fc: [Dword; 8],
    /// +0x11c (0x6350BC): `g_objectFirstFace`; now each view's (`ctx.view.object.first_face`); unused.
    pub object_first_face: Dword,
    /// +0x120 (0x6350C0): `g_objectFaceCount`; now each view's (`ctx.view.object.face_count`); unused.
    pub object_face_count: Dword,
    /// +0x124 (0x6350C4): the shape's first point, face, normal (`scene::add_object_shape`).
    pub shape_firsts: [Dword; 3],
    /// +0x130 (0x6350D0): not named.
    pub unknown_130: [Dword; 18],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl ObjectWork {
    /// The piece with the image's first values (a static of its own since 2026-10-29: it left
    /// the race block, phase 13, `race::store::LEFT`; the owner `scene::owner::Scene` holds it).
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x634FA0, the first byte of the piece.
pub const START: usize = 0x634FA0;
/// IDA 0x635118, the end of the piece.
pub const END: usize = 0x635118;

const _: () = assert!(END - START == core::mem::size_of::<ObjectWork>());

/// Where IDA 0x634FA0 is now (the piece's static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(std::ptr::from_ref(
        &crate::scene::owner::scene().drawing.object_work,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::owner::scene;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(scene().drawing.object_work.first(), START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(ObjectWork, face_cursor), 0x634FA0 - START);
        assert_eq!(offset_of!(ObjectWork, loading_car), 0x634FF0 - START);
        assert_eq!(offset_of!(ObjectWork, car_switches), 0x635028 - START);
        assert_eq!(offset_of!(ObjectWork, only_object), 0x635058 - START);
        assert_eq!(offset_of!(ObjectWork, light_level), 0x63508C - START);
    }
}
