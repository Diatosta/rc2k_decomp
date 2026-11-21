//! The object faces (in the race's buffers, `race::buffers_store`): 0x30000 bytes, written when
//! an object's shape is added (`scene::add_object_shape`), never while a drawing scope is open
//! (`session::DrawArc`, read through `FacesRef`).

/// The objects' faces (`scene::objects`, `scene::draw_object_faces`, `scene::list_object_faces`,
/// `race::frame_draw`).
#[derive(Clone)]
pub struct Faces {
    pub faces: [u32; 0xC000],
}

impl Faces {
    /// The faces, zeroed.
    pub const fn new() -> Self {
        Self { faces: [0; 0xC000] }
    }
}

/// The table as one reader finds it (the game's thread's own, or a drawing part's copy).
pub type FacesRef<'a> = crate::memory::drawn_table::DrawnTable<'a, Faces>;
