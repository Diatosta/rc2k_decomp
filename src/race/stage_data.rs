//! The data of the loaded stage that is not a plain cell store: the course's tables, the in-race
//! map, the mud, the objects' faces and points, and the best line worked out at the stage's load.
//! One value held by the course owner (`Course::stage`), taken when the stage loads and given back
//! when it is freed; every table is behind a lock or an atomic, so the drawing parts read it
//! through `CourseReads::stage` while the game's thread does not write.

use crate::memory::owned::Owned;
use crate::race::best_line::BestLine;
use crate::race::course_places::CourseData;
use crate::race::game_map::GameMap;
use crate::scene::object_faces::ObjectFaces;
use crate::scene::object_points::ObjectPoints;
use std::sync::Mutex;

/// The loaded stage's data.
pub struct StageData {
    /// The course's faces, records, scene points and info.
    pub course: CourseData,
    /// The in-race map's tiles and pixels.
    pub game_map: Owned<GameMap>,
    /// The mud file's strengths (0x10000 bytes).
    pub mud: Owned<Vec<u8>>,
    /// The objects' faces.
    pub object_faces: ObjectFaces,
    /// The objects' points and normals.
    pub object_points: ObjectPoints,
    /// The line `build_best_line` last worked out, kept for the race set-up to store again.
    pub best_line: Mutex<Option<BestLine>>,
}

impl StageData {
    /// A stage with nothing taken.
    pub const fn new() -> Self {
        StageData {
            course: CourseData::new(),
            game_map: Owned::new("no map tiles"),
            mud: Owned::new("no mud"),
            object_faces: ObjectFaces::new(),
            object_points: ObjectPoints::new(),
            best_line: Mutex::new(None),
        }
    }
}
