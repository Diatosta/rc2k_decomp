//! The race owner's course group (`race::owner::Race::course`): the course and the stage: course data and lists, level grid and paths, kind and light records, flares, marks settings, stage preferences, numbers, league.

use crate::race::course_lists_store::CourseLists;
use crate::race::course_store::CourseWork;
use crate::race::flares_store::FlareTables;
use crate::race::kind_records_store::KindRecords;
use crate::race::league_store::League;
use crate::race::level_grid_store::LevelGrid;
use crate::race::level_paths_store::LevelPaths;
use crate::race::lights_store::LightsData;
use crate::race::numbers_store::Numbers;
use crate::race::prefs_store::StagePrefs;

/// The race's course state (step 11, owned state: the stores move in as fields).
pub struct Course {
    /// One copy per drawing thread (`workers`).
    pub course: crate::workers::PerWorker<CourseWork>,
    /// One copy: the drawing only reads it (frame plan F4).
    pub lists: CourseLists,
    /// One copy: no drawing part writes it (the write watch's carried census, frame plan F7).
    pub level_grid: LevelGrid,
    pub level_paths: LevelPaths,
    pub kind_records: KindRecords,
    pub lights_data: LightsData,
    pub flare_tables: FlareTables,
    /// One copy (frame plan F4): the drawing writes only what each view's fog set-up works
    /// out from the draw distance (`race::draw_distance_settings`), the same in every view.
    pub stage_prefs: StagePrefs,
    pub numbers: Numbers,
    pub league: League,
}

impl Course {
    pub const fn new() -> Self {
        Course {
            course: crate::workers::PerWorker::new(
                [const { CourseWork::new() }; crate::workers::COPIES],
            ),
            lists: CourseLists::new(),
            level_grid: LevelGrid::new(),
            level_paths: LevelPaths::new(),
            kind_records: KindRecords::new(),
            lights_data: LightsData::new(),
            flare_tables: FlareTables::new(),
            stage_prefs: StagePrefs::new(),
            numbers: Numbers::new(),
            league: League::new(),
        }
    }
}
