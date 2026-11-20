//! An object's listed faces (`scene::draw_object`): its faces to draw, each 0x10 bytes (the
//! face's number within the object, the sort key, the polygon's third word, how
//! see-through it is), listed by `scene::list_object_face`, sorted by
//! `scene::sort_listed_faces` and drawn by `scene::draw_object_faces`. Each drawing has its
//! own list.

/// The listed faces, at most 0x1000 (`scene::list_object_face` refuses more).
pub type FaceList = Vec<rc2k_core::scene::face_order::ListedFace>;
