//! The race owner's course group (`race::owner::Race::course`): the course and the stage: course
//! data and lists, level grid and paths, kind and light records, flares, marks settings, stage
//! preferences, numbers, league.

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
use crate::session::{DrawShared, GameThread};

/// The race's course state.
pub struct Course {
    /// One copy, lent to the drawing parts (`workers::DrawReads`): no part writes it.
    pub course: DrawShared<CourseWork>,
    /// One copy, lent to the drawing parts (`workers::DrawReads`): no part writes it.
    pub lists: DrawShared<CourseLists>,
    /// One copy, lent to the drawing parts (`workers::DrawReads`): no part writes it.
    pub level_grid: DrawShared<LevelGrid>,
    /// One copy, lent to the drawing parts (`workers::DrawReads`): no part writes it (the fog
    /// settings the draw distance picks are set once a frame before the views,
    /// `race::draw_distance_settings`).
    pub stage_prefs: DrawShared<StagePrefs>,
    /// One copy, lent to the drawing parts (`workers::DrawReads`): no part writes it.
    pub numbers: DrawShared<Numbers>,
    /// One copy, lent to the drawing parts (`workers::DrawReads`): the ground search under a
    /// car's shadow reads it; no part writes it.
    kind_records: DrawShared<KindRecords>,
    main: GameThread<CourseMain>,
}

/// The course's drawing stores as one value, named as in `Course`: what code reached
/// both from the game's thread and from the drawing parts reads (the ground search under a
/// car's shadow, the course places, the scene points). `Course::reads` on the game's thread,
/// `workers::DrawReads::course` in a part.
#[derive(Clone, Copy)]
pub struct CourseReads<'a> {
    pub course: &'a CourseWork,
    pub lists: &'a CourseLists,
    pub stage_prefs: &'a StagePrefs,
    pub numbers: &'a Numbers,
    pub kind_records: &'a KindRecords,
}

/// What only the game's thread reaches (no drawing part and no other thread touches it).
struct CourseMain {
    level_paths: LevelPaths,
    lights_data: LightsData,
    flare_tables: FlareTables,
    league: League,
}

impl Course {
    pub const fn new() -> Self {
        Course {
            course: DrawShared::new(CourseWork::new()),
            lists: DrawShared::new(CourseLists::new()),
            level_grid: DrawShared::new(LevelGrid::new()),
            stage_prefs: DrawShared::new(StagePrefs::new()),
            numbers: DrawShared::new(Numbers::new()),
            kind_records: DrawShared::new(KindRecords::new()),
            main: GameThread::new(CourseMain {
                level_paths: LevelPaths::new(),
                lights_data: LightsData::new(),
                flare_tables: FlareTables::new(),
                league: League::new(),
            }),
        }
    }

    /// The drawing stores, on the game's thread outside a drawing scope.
    pub fn reads(&self) -> CourseReads<'_> {
        CourseReads {
            course: self.course.get(),
            lists: self.lists.get(),
            stage_prefs: self.stage_prefs.get(),
            numbers: self.numbers.get(),
            kind_records: self.kind_records.get(),
        }
    }

    pub fn level_paths(&self) -> &LevelPaths {
        &self.main.get().level_paths
    }

    /// The kinds' first and last records, on the game's thread outside a drawing scope (a
    /// part reads them through its `workers::DrawReads`).
    pub fn kind_records(&self) -> &KindRecords {
        self.kind_records.get()
    }

    /// The kinds' records as lent to a drawing scope (`workers::Lent`).
    pub(crate) fn kind_records_store(&self) -> &DrawShared<KindRecords> {
        &self.kind_records
    }

    pub fn lights_data(&self) -> &LightsData {
        &self.main.get().lights_data
    }

    pub fn flare_tables(&self) -> &FlareTables {
        &self.main.get().flare_tables
    }

    pub fn league(&self) -> &League {
        &self.main.get().league
    }
}
