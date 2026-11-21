//! The numbers of the course faces that are lit (a contact touched them, or a lamp burns): the
//! physics' step adds them, `drive::animate_lit_faces` ages them and the drawing parts gather the
//! lamps from them.

use crate::memory::session::{DrawPlain, PlainRead};

/// The most faces the list holds.
pub const LIST_ROOM: usize = rc2k_core::drive::lit_faces::LIST_ROOM as usize;

/// The list as a plain record (`session::DrawPlain`): written on the game's thread under short
/// borrows, copied for the drawing parts of a scope.
#[derive(Clone)]
pub struct LitFaces {
    /// The numbers of the course faces lit (`physics::touch_face`, `drive::animate_lit_faces`,
    /// `race::frame_draw::lights`).
    pub faces: [u32; LIST_ROOM],
    /// How many faces are in the list.
    pub count: u32,
}

/// The list, empty.
impl LitFaces {
    pub const fn new() -> Self {
        LitFaces {
            faces: [0; LIST_ROOM],
            count: 0,
        }
    }
}

/// The list as the game's thread holds it.
pub type LitFacesStore = DrawPlain<LitFaces>;

/// The list as one reader finds it: the game's thread's own (read through the store, short
/// borrows), or a drawing part's copy.
#[derive(Clone, Copy)]
pub enum LitFacesRef<'a> {
    Store(&'a LitFacesStore),
    Copy(&'a LitFaces),
}

impl LitFacesRef<'_> {
    /// How many faces are lit.
    pub fn count(self) -> u32 {
        match self {
            Self::Store(store) => store.borrow().count,
            Self::Copy(list) => list.count,
        }
    }

    /// The number of the lit face `k` (`k` below `LIST_ROOM`).
    pub fn face(self, k: usize) -> u32 {
        match self {
            Self::Store(store) => store.borrow().faces[k],
            Self::Copy(list) => list.faces[k],
        }
    }
}

impl<'a> From<&'a LitFacesStore> for LitFacesRef<'a> {
    fn from(store: &'a LitFacesStore) -> Self {
        Self::Store(store)
    }
}

impl<'a> From<&'a PlainRead<LitFaces>> for LitFacesRef<'a> {
    fn from(copy: &'a PlainRead<LitFaces>) -> Self {
        Self::Copy(copy)
    }
}
