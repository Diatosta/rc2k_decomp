//! The race owner's course group (`race::owner::Race::course`): the course and the stage: course
//! data and lists, level grid and paths, kind and light records, flares, marks settings, stage
//! preferences, numbers, league.

use crate::memory::session::{DrawArc, DrawPlain, GameThread};
use crate::race::course_lists_store::{CourseLists, CourseListsRef, CourseMemory, TurnWork};
use crate::race::course_store::{CourseWork, CourseWorkRef};
use crate::race::kind_records_store::{KindRecords, KindRecordsRef};
use crate::race::league_store::League;
use crate::race::level_paths_store::LevelPaths;
use crate::race::lights_store::{LightsData, LightsTable};
use crate::race::numbers_store::{NumberVars, NumberVarsRef, Numbers};
use crate::race::prefs_store::{StagePrefs, StagePrefsRef};
use crate::race::stage_data_store::StageData;
use std::cell::{Ref, RefCell, RefMut};

/// The race's course state.
pub struct Course {
    /// A copy is lent to the drawing parts (`scene::workers::DrawReads`): no part writes it.
    pub course: DrawPlain<CourseWork>,
    /// A copy is lent to the drawing parts (`scene::workers::DrawReads`): no part writes it.
    pub lists: DrawPlain<CourseLists>,
    /// Shared with the drawing parts (`scene::workers::DrawReads`): no part writes it (the fog
    /// settings the draw distance picks are set once a frame before the views,
    /// `race::draw_distance_settings`).
    pub stage_prefs: DrawArc<StagePrefs>,
    /// The game's float constants (a plain value, read by reference everywhere).
    pub numbers: Numbers,
    /// The replay's mode and `in_race`: a copy is lent to the drawing parts
    /// (`scene::workers::DrawReads`); no part writes it.
    pub number_vars: DrawPlain<NumberVars>,
    /// One copy, lent to the drawing parts (`scene::workers::DrawReads`): the ground search under a
    /// car's shadow reads it; no part writes it.
    kind_records: DrawPlain<KindRecords>,
    /// The loaded stage's tables (course, map, mud, objects, best line).
    pub stage: StageData,
    main: GameThread<CourseMain>,
}

/// The course's drawing stores as one value, named as in `Course`: what code reached
/// both from the game's thread and from the drawing parts reads (the ground search under a
/// car's shadow, the course places, the scene points). `Course::reads` on the game's thread,
/// `scene::workers::DrawReads::course` in a part.
#[derive(Clone, Copy)]
pub struct CourseReads<'a> {
    pub course: CourseWorkRef<'a>,
    pub lists: CourseListsRef<'a>,
    pub stage_prefs: StagePrefsRef<'a>,
    pub numbers: &'a Numbers,
    pub number_vars: NumberVarsRef<'a>,
    pub kind_records: KindRecordsRef<'a>,
    pub stage: &'a StageData,
}

/// What only the game's thread reaches (no drawing part and no other thread touches it).
struct CourseMain {
    /// The grip forces' sum's change, eased to 0 (`physics::drive_grip_forces`): a cell, since
    /// that code loads and stores it as an operand.
    grip_change: crate::memory::session::PlainFloat,
    memory: CourseMemory,
    turn_work: TurnWork,
    level_paths: RefCell<LevelPaths>,
    lights_data: LightsData,
    lights_table: RefCell<LightsTable>,
    league: RefCell<League>,
}

impl Course {
    pub const fn new() -> Self {
        Course {
            course: DrawPlain::new(CourseWork::new()),
            lists: DrawPlain::new(CourseLists::new()),
            stage_prefs: DrawArc::new(StagePrefs::new),
            numbers: Numbers::new(),
            number_vars: DrawPlain::new(NumberVars::new()),
            kind_records: DrawPlain::new(KindRecords::new()),
            stage: StageData::new(),
            main: GameThread::new(CourseMain {
                grip_change: crate::memory::session::PlainFloat::new(0.0),
                memory: CourseMemory::new(),
                turn_work: TurnWork::new(),
                level_paths: RefCell::new(LevelPaths::new()),
                lights_data: LightsData::new(),
                lights_table: RefCell::new(LightsTable::new()),
                league: RefCell::new(League::new()),
            }),
        }
    }

    /// The drawing stores, on the game's thread outside a drawing scope.
    pub fn reads(&self) -> CourseReads<'_> {
        CourseReads {
            course: CourseWorkRef::Store(&self.course),
            lists: CourseListsRef::Store(&self.lists),
            stage_prefs: StagePrefsRef::Store(&self.stage_prefs),
            numbers: &self.numbers,
            number_vars: NumberVarsRef::Store(&self.number_vars),
            kind_records: KindRecordsRef::Store(&self.kind_records),
            stage: &self.stage,
        }
    }

    /// Whether the race is on.
    pub fn in_race(&self) -> bool {
        self.number_vars.borrow().in_race
    }

    pub fn set_in_race(&self, on: bool) {
        self.number_vars.borrow_mut().in_race = on;
    }

    /// The replay's mode: 1 recording, 2 playing.
    pub fn replay_mode(&self) -> u32 {
        self.number_vars.borrow().replay_mode
    }

    pub fn set_replay_mode(&self, mode: u32) {
        self.number_vars.borrow_mut().replay_mode = mode;
    }

    /// The grip forces' sum's change.
    pub fn grip_change(&self) -> &crate::memory::session::PlainFloat {
        &self.main.get().grip_change
    }

    /// The course's memory lists.
    pub fn memory(&self) -> &CourseMemory {
        &self.main.get().memory
    }

    /// The physics' turn and angle work values.
    pub fn turn_work(&self) -> &TurnWork {
        &self.main.get().turn_work
    }

    /// The level and picture paths, read.
    pub fn level_paths(&self) -> Ref<'_, LevelPaths> {
        self.main.get().level_paths.borrow()
    }

    /// The level and picture paths, to write: no other borrow may be held.
    pub fn level_paths_mut(&self) -> RefMut<'_, LevelPaths> {
        self.main.get().level_paths.borrow_mut()
    }

    /// The kinds' first and last records, on the game's thread outside a drawing scope (a
    /// part reads them through its `scene::workers::DrawReads`; a copy of them is lent to a
    /// drawing scope by `scene::workers::Lent`).
    pub fn kind_records(&self) -> &DrawPlain<KindRecords> {
        &self.kind_records
    }

    pub fn lights_data(&self) -> &LightsData {
        &self.main.get().lights_data
    }

    /// The lights' data read from the stage's file, read: no write borrow may be held.
    pub fn lights_table(&self) -> Ref<'_, LightsTable> {
        self.main.get().lights_table.borrow()
    }

    /// The lights' data, to write: no other borrow may be held.
    pub fn lights_table_mut(&self) -> RefMut<'_, LightsTable> {
        self.main.get().lights_table.borrow_mut()
    }

    /// The league's kept times, read: no write borrow may be held.
    pub fn league(&self) -> Ref<'_, League> {
        self.main.get().league.borrow()
    }

    /// The league's kept times, to write: no other borrow may be held.
    pub fn league_mut(&self) -> RefMut<'_, League> {
        self.main.get().league.borrow_mut()
    }
}
