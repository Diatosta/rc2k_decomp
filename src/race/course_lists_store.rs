//! The course memory's lists, the object and point-record pointers, the face-key factors, the two
//! root tables and the physics' turn work. Three pieces: `CourseLists`, a plain record
//! (`session::DrawPlain`) written on the game's thread between frames and read there and, as a
//! copy made when a drawing scope opens, by the drawing parts; `CourseMemory`, the course's memory
//! lists (the allocators take them as `ListCells`), and `TurnWork`, the physics' matrix and angle
//! work values (the float helpers take cells as operands); both only the game's thread reaches.

use crate::memory::ListCells;
use crate::memory::session::{DrawPlain, PlainFloat, PlainRead};

/// The values the drawing parts read and the course's allocators keep.
#[derive(Clone)]
pub struct CourseLists {
    /// The mask of input keys read for the car's controls (`drive::read_car_controls`).
    pub input_keys_mask: u32,
    /// Floats for the face sort key (`scene::face_sort_key`, `scene::fill_engine_tables`).
    pub face_key_factors: [f32; 256],
    /// Bytes taken from the course lists, added and subtracted by the allocators
    /// (`race::stage_free`, `race::mud`, `scene::allocate_object_memory`).
    pub course_bytes_total: u32,
    /// Set when the second course list is freed (cleared by `race::stage_free`).
    pub second_course_list_a: u32,
    /// As its neighbour (cleared by `race::stage_free`).
    pub second_course_list_b: u32,
    /// The object points' memory (`scene::objects`, `scene::place_object_points`,
    /// `physics::object_hits_car`).
    pub object_points: u32,
    /// The faces' memory (`scene::objects`, `race::stage_free`).
    pub face_memory: u32,
    /// Bytes left for the object points (`scene::objects`).
    pub object_points_room: u32,
    /// Bytes left for the faces (`scene::objects`).
    pub face_memory_room: u32,
    /// The point records' memory (`scene::project_point`, `race::frame_draw`,
    /// `race::car_shadow`).
    pub point_records: u32,
    /// Square roots by mantissa bits (`math::normalise_vector`, `scene::fill_engine_tables`,
    /// `race::frame_draw`).
    pub root_table: [f32; 256],
    /// Roots by exponent (same users).
    pub exponent_root_table: [f32; 256],
}

impl CourseLists {
    pub const fn new() -> Self {
        Self {
            input_keys_mask: 0,
            face_key_factors: [0.0; 256],
            course_bytes_total: 0,
            second_course_list_a: 0,
            second_course_list_b: 0,
            object_points: 0,
            face_memory: 0,
            object_points_room: 0,
            face_memory_room: 0,
            point_records: 0,
            root_table: [0.0; 256],
            exponent_root_table: [0.0; 256],
        }
    }
}

/// The lists as one reader finds them: the game's thread's own (read through the store when
/// used), or a drawing part's copy.
#[derive(Clone, Copy)]
pub enum CourseListsRef<'a> {
    Store(&'a DrawPlain<CourseLists>),
    Copy(&'a CourseLists),
}

impl CourseListsRef<'_> {
    /// `f` on the lists as they stand (a short read: nothing in `f` may write the store).
    pub fn read<R>(self, f: impl FnOnce(&CourseLists) -> R) -> R {
        match self {
            Self::Store(store) => f(&store.borrow()),
            Self::Copy(lists) => f(lists),
        }
    }

    /// The point records' memory.
    pub fn point_records(self) -> u32 {
        self.read(|lists| lists.point_records)
    }

    /// The object points' memory.
    pub fn object_points(self) -> u32 {
        self.read(|lists| lists.object_points)
    }

    /// Face-key factor `n` (refused past the table by the caller's index check).
    pub fn face_key_factor(self, n: usize) -> f32 {
        self.read(|lists| lists.face_key_factors[n])
    }
}

impl<'a> From<&'a PlainRead<CourseLists>> for CourseListsRef<'a> {
    fn from(copy: &'a PlainRead<CourseLists>) -> Self {
        Self::Copy(copy)
    }
}

/// The course's memory lists.
pub struct CourseMemory {
    /// The course's MemoryList of 16 bytes (`race::level_init::allocate_course_memory`,
    /// `race::stage_free`).
    pub course_memory_list: ListCells,
    /// The second course MemoryList: set up by `race::level_init::allocate_course_memory` and
    /// freed by `race::stage_free`; nothing allocates from it.
    pub second_course_list: ListCells,
    /// The object points' list (`scene::allocate_object_memory`).
    pub object_points_list: ListCells,
    /// The point records' list (`scene::allocate_point_records`).
    pub point_records_list: ListCells,
    /// The mud's list (`race::mud`).
    pub mud_list: ListCells,
}

impl CourseMemory {
    pub const fn new() -> Self {
        Self {
            course_memory_list: ListCells::empty(),
            second_course_list: ListCells::empty(),
            object_points_list: ListCells::empty(),
            point_records_list: ListCells::empty(),
            mud_list: ListCells::empty(),
        }
    }
}

/// The physics' work values of a turn: the turn matrix and the spin's axis and angle
/// (`physics::move_slot`, `physics::spin_matrix`, `physics::axis_turn_matrix`), the turn's
/// sines and products, and the car's angles with the turned unit vector
/// (`physics::matrix_angles`, `physics::car_angles`).
pub struct TurnWork {
    pub turn_matrix: [PlainFloat; 16],
    pub turn_sine: PlainFloat,
    pub turn_cosine: PlainFloat,
    pub turn_one_less_cosine: PlainFloat,
    /// (1 - cosine) x z.
    pub turn_product_xz: PlainFloat,
    /// (1 - cosine) x y.
    pub turn_product_xy: PlainFloat,
    /// (1 - cosine) y z.
    pub turn_product_yz: PlainFloat,
    pub turn_sine_x: PlainFloat,
    pub turn_sine_y: PlainFloat,
    pub turn_sine_z: PlainFloat,
    /// The car's pitch: minus the clamped asin of the turned forward vector's y.
    pub car_pitch: PlainFloat,
    /// The car's heading: atan2 of the turned forward vector's x over z.
    pub car_heading: PlainFloat,
    /// The car's roll: asin of the turned side vector's y over the pitch's cosine.
    pub car_roll: PlainFloat,
    pub turned_x: PlainFloat,
    pub turned_y: PlainFloat,
    pub turned_z: PlainFloat,
    /// The angle of the turn.
    pub spin_angle: PlainFloat,
    /// The axis' x, y, z.
    pub spin_axis: [PlainFloat; 3],
}

impl TurnWork {
    pub const fn new() -> Self {
        Self {
            turn_matrix: [const { PlainFloat::new(0.0) }; 16],
            turn_sine: PlainFloat::new(0.0),
            turn_cosine: PlainFloat::new(0.0),
            turn_one_less_cosine: PlainFloat::new(0.0),
            turn_product_xz: PlainFloat::new(0.0),
            turn_product_xy: PlainFloat::new(0.0),
            turn_product_yz: PlainFloat::new(0.0),
            turn_sine_x: PlainFloat::new(0.0),
            turn_sine_y: PlainFloat::new(0.0),
            turn_sine_z: PlainFloat::new(0.0),
            car_pitch: PlainFloat::new(0.0),
            car_heading: PlainFloat::new(0.0),
            car_roll: PlainFloat::new(0.0),
            turned_x: PlainFloat::new(0.0),
            turned_y: PlainFloat::new(0.0),
            turned_z: PlainFloat::new(0.0),
            spin_angle: PlainFloat::new(0.0),
            spin_axis: [const { PlainFloat::new(0.0) }; 3],
        }
    }
}
