//! The scene objects' work: the objects' loading (memory used, the kind and slot being loaded, a
//! shape's counts and firsts), the object being drawn (its place, position, size, angles, faces)
//! and the light levels.

use crate::memory::cells::ZeroCells;
use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ObjectWork {
    /// The bytes of face memory used (`scene::objects`).
    pub face_cursor: PlainDword,
    /// The bytes of point memory used (the log's "total pointmem";
    /// `scene::load_front_end_objects`).
    pub point_memory_total: PlainDword,
    /// The bytes of face memory used (the log's "total facemem").
    pub face_memory_total: PlainDword,
    /// Eight dwords between the memory totals and the shape's counts; no code uses them (a
    /// gap).
    pub unknown_c: [PlainDword; 8],
    /// The shape's counts (`scene::add_object_shape`): [0] faces, [1] points, [3] normals, [5]
    /// lowest, [6] highest.
    pub shape_counts: [PlainDword; 7],
    /// The kind of the object being loaded.
    pub loading_kind: PlainDword,
    /// The slot being loaded.
    pub loading_slot: PlainDword,
    /// The car being loaded, its part, the car's first slot (`scene::load_object`; [1] read by
    /// `scene::add_object_shape`).
    pub loading_car: [PlainDword; 3],
    /// The scene point of the object being drawn, -1 none (`scene::draw_described_object`,
    /// `scene::set_object_fade`). Unused: each view has its own in
    /// `scene::object_place::ObjectPlace`.
    pub object_place: PlainDword,
    /// One dword between the object's place and its position; no code uses it (a gap).
    pub unknown_60: PlainDword,
    /// The object's x, y, depth (`scene::set_object_fade`, `physics::fill_race_slot`); the
    /// drawing's is each view's (`ctx.view.object.position`); this one is
    /// `physics::fill_race_slot`'s own.
    pub object_position: [PlainDword; 3],
    /// The face depth (`race::frame_draw`). Unused: each view has its own in
    /// `ctx.view.object.face_depth`.
    pub face_depth: PlainDword,
    /// Cleared by `race::draw_view_scene`. Unused: each view has its own in
    /// `ObjectPlace::view_cleared`.
    pub view_cleared: [PlainDword; 2],
    /// The matrix's place, x y z (`scene::matrix`). Read by none: each view has its own in
    /// `ctx.view.object.place`.
    pub place: [PlainDword; 3],
    /// The car's switches (`race::car_draw`). Unused: each view has its own in
    /// `ctx.view.object.car_switches`.
    pub car_switches: PlainDword,
    /// The place turned (`race::frame_draw`). Unused: each view has its own in
    /// `ctx.view.object.turned_place`.
    pub turned_place: [PlainDword; 3],
    /// The object's size (`scene::matrix`); each view's drawing uses `ctx.view.object.size`;
    /// this one is the physics' contact tests' own.
    pub size: PlainDword,
    /// The object's angles, x y z: the physics' (`physics::object_turn_matrix`); the drawing's
    /// are the view's (`scene::object_place`).
    pub angles: [PlainDword; 3],
    /// The car's part values (`race::car_parts`).
    pub part_values: [PlainDword; 3],
    /// The other light of the object being drawn. Unused: each view has its own in
    /// `scene::object_light`.
    pub other_light: PlainDword,
    /// The described object's mode bits (`scene::draw_shown_object`). Unused: each view has its
    /// own in `scene::object_place::ObjectPlace`.
    pub only_object: PlainDword,
    /// Cleared by `scene::draw_described_object`. Unused: each view has its own in
    /// `ObjectPlace::described_turns`.
    pub described_cleared: [PlainDword; 3],
    /// The light added to each vertex (`scene::vertex_light`). Unused: each view has its own in
    /// `scene::object_light`.
    pub light_added: PlainDword,
    /// Two dwords between the light added and the part weights; no code uses them (a gap).
    pub unknown_cc: [PlainDword; 2],
    /// The car's part weights (`race::car_parts`).
    pub part_weights: [PlainDword; 4],
    /// The lit colour's lowest place. Unused: each view has its own in `scene::object_light`.
    pub light_lowest: PlainDword,
    /// The lit colour's highest place. Unused: each view has its own in `scene::object_light`.
    pub light_highest: PlainDword,
    /// The lit colour's level. Unused: each view has its own in `scene::object_light`.
    pub light_level: PlainDword,
    /// The light's strength. Read by none: each view has its own in
    /// `ctx.view.object.light_strength`.
    pub light_strength: PlainDword,
    /// The light's turns, y then x (`scene::matrix`). Unused: each view has its own in
    /// `scene::object_light`.
    pub light_turns: [PlainDword; 2],
    /// Eight dwords after the light's turns; no code uses them (a gap).
    pub unknown_fc: [PlainDword; 8],
    /// The object's first face. Unused: each view has its own in `ctx.view.object.first_face`.
    pub object_first_face: PlainDword,
    /// The object's face count. Unused: each view has its own in `ctx.view.object.face_count`.
    pub object_face_count: PlainDword,
    /// The shape's first point, face, normal (`scene::add_object_shape`).
    pub shape_firsts: [PlainDword; 3],
    /// Eighteen dwords closing the piece; nothing uses them (a gap).
    pub unknown_130: [PlainDword; 18],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl ObjectWork {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self {
            object_place: PlainDword::new(0xFFFFFFFF),
            ..ZeroCells::ZERO
        }
    }
}
