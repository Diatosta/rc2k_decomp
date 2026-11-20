//! The scene objects' work: the objects' loading (memory used, the kind and slot being loaded, a
//! shape's counts and firsts) and the values the physics hands the object drawn (position, size,
//! angles, car part values and weights). Plain fields: only the game's thread reaches them
//! (`scene::owner::Scene::object_work`).

/// The scene objects' work variables.
#[derive(Clone, Copy, Default)]
pub struct ObjectWork {
    /// The bytes of face memory used (`scene::objects`).
    pub face_cursor: u32,
    /// The bytes of point memory used (the log's "total pointmem";
    /// `scene::load_front_end_objects`).
    pub point_memory_total: u32,
    /// The bytes of face memory used (the log's "total facemem").
    pub face_memory_total: u32,
    /// The shape's counts (`scene::add_object_shape`): [0] faces, [1] points, [3] normals, [5]
    /// lowest, [6] highest.
    pub shape_counts: [u32; 7],
    /// The kind of the object being loaded.
    pub loading_kind: u32,
    /// The slot being loaded.
    pub loading_slot: u32,
    /// The car being loaded, its part, the car's first slot (`scene::load_object`; [1] read by
    /// `scene::add_object_shape`).
    pub loading_car: [u32; 3],
    /// The object's x, y, depth: `physics::fill_race_slot`'s own (the drawing's is each view's,
    /// `ctx.view.object.position`).
    pub object_position: [u32; 3],
    /// Cleared by `race::draw_view_scene`: each view has its own in
    /// `ObjectPlace::view_cleared`; only the developers' watch row reads this one.
    pub view_cleared: [u32; 2],
    /// The object's size: the physics' contact tests' own (the drawing's is `ctx.view.object.size`).
    pub size: u32,
    /// The object's angles, x y z: the physics' (`physics::object_turn_matrix`); the drawing's
    /// are the view's (`scene::object_place`).
    pub angles: [u32; 3],
    /// The car's part values (`race::car_parts`).
    pub part_values: [u32; 3],
    /// The car's part weights (`race::car_parts`).
    pub part_weights: [u32; 4],
    /// The shape's first point, face, normal (`scene::add_object_shape`).
    pub shape_firsts: [u32; 3],
}

impl ObjectWork {
    /// The store with its first values (every field starts at zero).
    pub const fn new() -> Self {
        Self {
            face_cursor: 0,
            point_memory_total: 0,
            face_memory_total: 0,
            shape_counts: [0; 7],
            loading_kind: 0,
            loading_slot: 0,
            loading_car: [0; 3],
            object_position: [0; 3],
            view_cleared: [0; 2],
            size: 0,
            angles: [0; 3],
            part_values: [0; 3],
            part_weights: [0; 4],
            shape_firsts: [0; 3],
        }
    }
}
