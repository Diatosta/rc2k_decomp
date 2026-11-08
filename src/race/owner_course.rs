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
    /// One copy per drawing thread (`workers`).
    pub lists: crate::workers::PerWorker<CourseLists>,
    /// One copy per drawing thread (`workers`).
    pub level_grid: crate::workers::PerWorker<LevelGrid>,
    pub level_paths: LevelPaths,
    pub kind_records: KindRecords,
    pub lights_data: LightsData,
    pub flare_tables: FlareTables,
    /// One copy per drawing thread (`workers`).
    pub stage_prefs: crate::workers::PerWorker<StagePrefs>,
    pub numbers: Numbers,
    pub league: League,
}

impl Course {
    pub const fn new() -> Self {
        Course {
            course: crate::workers::PerWorker::new(
                [const { CourseWork::new() }; crate::workers::COPIES],
            ),
            lists: crate::workers::PerWorker::new(
                [const { CourseLists::new() }; crate::workers::COPIES],
            ),
            level_grid: crate::workers::PerWorker::new(
                [const { LevelGrid::new() }; crate::workers::COPIES],
            ),
            level_paths: LevelPaths::new(),
            kind_records: KindRecords::new(),
            lights_data: LightsData::new(),
            flare_tables: FlareTables::new(),
            stage_prefs: crate::workers::PerWorker::new(
                [const { StagePrefs::new() }; crate::workers::COPIES],
            ),
            numbers: Numbers::new(),
            league: League::new(),
        }
    }
}
