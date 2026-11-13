//! The drawing's threads (the optimisation phase's multi-core drawing, `notes/conversion.md`):
//! parts of a frame (the players' views and their mirrors) drawn by several threads at once,
//! so the frame is what one thread draws. Each view draws on its own working state
//! (`race::view_scratch`); what a view hands on to the shared stores is applied on the game's
//! thread in the views' order (`race::draw_race_scene`'s `view_drawn`), and the render
//! states' cache and the held matrix columns a part keeps are put back in the parts' order
//! (frame plan F7: no drawing part writes a shared store).
//!
//! The scratch stores each part makes for itself before reading (the polygon vertex buffer,
//! the polygon work, the point records) are a `PartScratch`: one per part (`part_scratch`),
//! handed to the part's drawing and passed down to what reaches them. Part 0's is the game's
//! own thread's, what everything drawn outside the parts works on too (`game_scratch`).
//! A `PartScratch` is made of plain cells (and the `RefCell` of its noted drawing, which only
//! the thread that has claimed the part reaches): it is `Send`, asserted below.
//!
//! Inside a part, one piece of its drawing can be drawn aside on another drawing thread
//! (`draw_aside`, the view's own car: `race::car_part`), on a scratch of its own behind a
//! mutex, from the part's render states as the piece was handed aside; its drawing is put in
//! at its place in the part's order, as `draw_parts` puts a part's after the earlier ones.

use crate::errors::{BadInput, Refused};
use crate::part_cell::{PartCell, PartTurn};
use crate::race::view_scratch::ViewScratch;
use std::cell::Cell;
use std::sync::{Condvar, Mutex, OnceLock};

/// The most drawing threads, the game's own one with them.
pub const COUNT: usize = 4;

thread_local! {
    /// The running thread's copy: 0 unless it is one of the drawing threads.
    static INDEX: Cell<usize> = const { Cell::new(0) };
}

/// The running thread's copy number.
#[inline(always)]
pub fn index() -> usize {
    INDEX.get()
}

/// What a drawing part makes for itself before reading (see the module's header): neither
/// copied in nor merged, each part's work on its own (the replays' every flush the same as on
/// one thread).
pub struct PartScratch {
    /// The polygon vertex buffer (`d3d::vertices_store`).
    pub vertices: crate::d3d::vertices_store::PolygonVertices,
    /// The polygon work: the entries, their lists, the clipper's pool
    /// (`scene::polygon_store`).
    pub polygon: crate::scene::polygon_store::PolygonWork,
    /// The point records, a cache of the points' projections (`scene::point_records`).
    pub records: crate::scene::point_records::Records,
    /// Which records have been written since they were last forgotten
    /// (`scene::point_records::Written`).
    pub records_written: crate::scene::point_records::Written,
    /// The held matrix columns while a part other than the first is drawn
    /// (`scene::view_matrices::Held`), from the list's start.
    pub columns: crate::scene::view_matrices::OwnColumns,
    /// The scene points past the count while the part is drawn in a run
    /// (`race::scene_points::OwnPoints`).
    pub points: crate::race::scene_points::OwnPoints,
    /// The render states' and the current texture's cache (`d3d::render_cache::OwnCache`):
    /// part 0's is always in use, the others' while their run is drawn.
    pub cache: crate::d3d::render_cache::OwnCache,
    /// Where such a part first set each state (`d3d::run_states::OwnRun`).
    pub run: crate::d3d::run_states::OwnRun,
    /// The drawing it has noted (`platform_3d::Noted`): the first part's what the game's
    /// thread flushes, the others' put after it at their runs' ends. Behind a mutex: it is
    /// reached by the game's thread (`platform_3d::game_part`) as well as by the part's own.
    pub noted: crate::platform_3d::Noted,
    /// Whether a race is on (`race::numbers_store::Numbers::in_race`) as the parts of a
    /// drawing scope read it, set from the store before the scope opens (`draw_parts`): the
    /// GPU draws only then. Read through `in_race`.
    pub in_race: crate::session::PlainFlag,
    /// The screen's width, height and bits per pixel (`session::Screen`) as the parts of a
    /// drawing scope read them for the noted drawing, set from the store before the scope
    /// opens (`draw_parts`), as `in_race` is. Read by `platform_3d`.
    pub screen_size: [crate::session::PlainDword; 3],
}

impl PartScratch {
    /// The scratch of part 0 (`first`: its render cache in use from the start) or another
    /// part, zeroed as the image's tail was.
    const fn new(first: bool) -> Self {
        Self {
            vertices: crate::d3d::vertices_store::PolygonVertices::new(),
            polygon: crate::scene::polygon_store::PolygonWork::new(),
            records: crate::scene::point_records::new_records(),
            records_written: crate::scene::point_records::new_written(),
            columns: crate::scene::view_matrices::OwnColumns::new(),
            points: crate::race::scene_points::OwnPoints::new(),
            cache: if first {
                crate::d3d::render_cache::OwnCache::first()
            } else {
                crate::d3d::render_cache::OwnCache::new()
            },
            run: crate::d3d::run_states::OwnRun::new(),
            noted: crate::platform_3d::Noted::new(),
            in_race: crate::session::PlainFlag::new(false),
            screen_size: [const { crate::session::PlainDword::new(0) }; 3],
        }
    }
}

// A part's scratch is handed from one thread to the next (`PartCell`'s proof needs `Send`).
const _: () = {
    const fn assert_send<T: Send>() {}
    assert_send::<PartScratch>();
};

/// The parts' scratch. Run `n` of a drawing scope claims element `n` (`draw_parts`); the
/// game's thread reaches element 0 by `part_scratch` and `game_scratch`, the others (all
/// four, with the scope closed) by `all_parts_on_game_thread`.
static PARTS: [PartCell<PartScratch>; COUNT] = [
    PartCell::new(PartScratch::new(true)),
    PartCell::new(PartScratch::new(false)),
    PartCell::new(PartScratch::new(false)),
    PartCell::new(PartScratch::new(false)),
];

/// Every part's scratch, on the game's thread with no drawing scope open (so no run holds
/// one): what a clear of the point records reaches.
pub fn all_parts_on_game_thread() -> impl Iterator<Item = &'static PartScratch> {
    debug_assert!(
        !in_scope(),
        "every part's scratch reached inside a drawing scope"
    );
    PARTS.iter().map(PartCell::on_game_thread)
}

/// The first part's scratch, on the game's thread (inside a drawing scope, run 0 is that
/// thread): what the platform's flush and the render states' cache reach it by. `n` is 0:
/// the other parts' are handed to their runs (`PartRun`).
pub fn part_scratch(n: usize) -> &'static PartScratch {
    debug_assert_eq!(n, 0, "only the first part's scratch is reached by number");
    PARTS[n].on_game_thread()
}

/// The game's own thread's scratch: part 0's, what everything drawn outside the parts (the
/// overlays, the HUD, the front end) works on.
pub fn game_scratch() -> &'static PartScratch {
    debug_assert_eq!(index(), 0, "the game's scratch on a drawing part");
    PARTS[0].on_game_thread()
}

// ---- the stores the parts read ----

/// Whether a drawing scope is open (`DrawScope`): the parts are being drawn.
static IN_SCOPE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether the parts of a drawing are being drawn: between `draw_parts`' start of the parts
/// and the end of the last (`session::DrawShared::get` is not taken then).
pub fn in_scope() -> bool {
    IN_SCOPE.load(std::sync::atomic::Ordering::Relaxed)
}

// r40course
/// Whether a race is on (`race::numbers_store::Numbers::in_race`), as part `part`'s drawing
/// reads it: inside a drawing scope its copy taken when the scope opened, outside one the
/// store (on the game's thread).
pub fn in_race(part: &PartScratch) -> bool {
    if in_scope() {
        part.in_race.is_set()
    } else {
        // m6-root: read by the drawing workers' platform callback.
        crate::race::owner::RACE // m6-root
            .course
            .numbers
            .get()
            .in_race
            .is_set()
    }
}
// end r40course

/// The parts of one drawing being drawn (`draw_parts`): opened on the game's thread before
/// the first part starts, ended once every part has ended. The stores lent to the parts
/// (`session::PhaseRead`) borrow it, so none outlives it. Only `draw_parts` opens one.
pub struct DrawScope {
    _only_here: (),
}

impl DrawScope {
    fn open() -> Self {
        crate::session::assert_game_thread();
        let was = IN_SCOPE.swap(true, std::sync::atomic::Ordering::Relaxed);
        debug_assert!(!was, "a drawing scope opened inside another");
        Self { _only_here: () }
    }
}

impl Drop for DrawScope {
    fn drop(&mut self) {
        crate::session::end_lending();
        for part in &PARTS {
            part.end_claim(self);
        }
        crate::race::view_scratch::end_claims(self);
        IN_SCOPE.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

/// The owners whose class D stores a drawing reads: lent to the parts of a drawing scope
/// (`draw_parts`), or read from on the game's thread (`DrawReads::on_game_thread`).
#[derive(Clone, Copy)]
pub struct LentOwners<'a> {
    pub race: &'a crate::race::owner::Race,
    pub scene: &'a crate::scene::owner::Scene,
    pub system: &'a crate::system::owner::System,
}

impl<'a> LentOwners<'a> {
    /// The race's and the scene's owners of `o`.
    pub fn of(o: crate::owners::Owners<'a>) -> Self {
        Self {
            race: o.race,
            scene: o.scene,
            system: o.system,
        }
    }
}

/// The class D stores (`notes/m3_census.md`, section 2b) lent to the parts of one drawing
/// scope: what every part reads and none writes. Shared by reference with every part's run;
/// a part reads them through `reads`.
pub struct Lent<'s> {
    light_tables: crate::session::PhaseRead<'s, crate::scene::light_tables_store::LightTables>,
    // The scene's other drawing stores.
    light_work: crate::session::PhaseRead<'s, crate::scene::light_work_store::LightWork>,
    draw_switches: crate::session::PhaseRead<'s, crate::scene::draw_switches_store::DrawSwitches>,
    object_tables: crate::session::PhaseRead<'s, crate::scene::object_tables_store::ObjectTables>,
    object_slots: crate::session::PhaseRead<'s, crate::scene::object_slots_store::ObjectTables>,
    kind_textures:
        crate::session::PhaseRead<'s, crate::scene::kind_texture_store::KindTextureTables>,
    texture_places:
        crate::session::PhaseRead<'s, crate::scene::texture_places_store::TexturePlaces>,
    light_state: crate::session::PhaseRead<'s, crate::scene::light_store::LightState>,
    // The scene's draw variables.
    scene_draw: crate::session::PhaseRead<'s, crate::scene::draw_store::SceneDraw>,
    // The race's stores.
    level_grid: crate::session::PhaseRead<'s, crate::race::level_grid_store::LevelGrid>,
    ground_probe: crate::session::PhaseRead<'s, crate::physics::ground_probe_store::GroundProbe>,
    shadow_skids: crate::session::PhaseRead<'s, crate::race::shadow_skids_store::ShadowSkids>,
    lights_and_gear: crate::session::PhaseRead<'s, crate::race::groups_store::LightsAndGear>,
    finish: crate::session::PhaseRead<'s, crate::drive::finish_store::Finish>,
    // The frame's work and the panel and overlay.
    frame_work: crate::session::PhaseRead<'s, crate::race::frame_store::FrameWork>,
    panel_and_overlay: crate::session::PhaseRead<'s, crate::race::groups_store::PanelAndOverlay>,
    // The cars' blocks and the view variables.
    car_blocks: crate::session::PhaseRead<'s, crate::race::blocks_store::CarBlocks>,
    view_and_records: crate::session::PhaseRead<'s, crate::race::groups_store::ViewAndRecords>,
    // The system's screen and the processor's MMX flag.
    screen: crate::session::PhaseRead<'s, crate::session::Screen>,
    has_mmx: bool,
    // The course's groups.
    course_lists: crate::session::PhaseRead<'s, crate::race::course_lists_store::CourseLists>,
    stage_prefs: crate::session::PhaseRead<'s, crate::race::prefs_store::StagePrefs>,
    course_work: crate::session::PhaseRead<'s, crate::race::course_store::CourseWork>,
    kind_records: crate::session::PhaseRead<'s, crate::race::kind_records_store::KindRecords>,
    numbers: crate::session::PhaseRead<'s, crate::race::numbers_store::Numbers>,
    // The race owner's shared group (lane r41shared).
    switches: crate::session::PhaseRead<'s, crate::race::switches_store::FrameSwitches>,
    vars: crate::session::PhaseRead<'s, crate::race::vars_store::RaceVars>,
    view: crate::session::PhaseRead<'s, crate::race::view_store::RaceView>,
    work: crate::session::PhaseRead<'s, crate::race::work_store::RaceWork>,
    physics_tables: crate::session::PhaseRead<'s, crate::physics::tables_store::PhysicsTables>,
    // The frame's matrices and the physics' contact work.
    matrices: crate::session::PhaseRead<'s, crate::race::matrices_store::SceneMatrices>,
    contact: crate::session::PhaseRead<'s, crate::physics::contact_store::ContactWork>,
    // The race's buffers the parts read.
    buffers: crate::session::PhaseRead<'s, crate::race::buffers_store::BufferReads>,
}

/// Whether the processor has MMX and the game uses it: fixed at start-up, so one value for
/// every part of a drawing.
fn has_mmx(system: &crate::system::owner::System) -> bool {
    system.machine().has_mmx.get() != 0
}

impl<'s> Lent<'s> {
    fn new(scope: &'s DrawScope, owners: LentOwners<'s>) -> Self {
        use crate::session::PhaseRead;
        let LentOwners {
            race,
            scene,
            system,
        } = owners;
        Self {
            light_tables: PhaseRead::lend(&scene.light_tables, scope),
            // The scene's other drawing stores.
            light_work: PhaseRead::lend(&scene.lights.work, scope),
            draw_switches: PhaseRead::lend(&scene.draw_switches, scope),
            object_tables: PhaseRead::lend(&scene.objects.tables, scope),
            object_slots: PhaseRead::lend(&scene.objects.slots, scope),
            kind_textures: PhaseRead::lend(&scene.textures.kinds, scope),
            texture_places: PhaseRead::lend(&scene.textures.places, scope),
            light_state: PhaseRead::lend(&scene.lights.state, scope),
            // The scene's draw variables.
            scene_draw: PhaseRead::lend(&scene.drawing.scene, scope),
            // The race's stores.
            level_grid: PhaseRead::lend(&race.course.level_grid, scope),
            ground_probe: PhaseRead::lend(&race.physics.ground_probe, scope),
            shadow_skids: PhaseRead::lend(&race.frame.shadow_skids, scope),
            lights_and_gear: PhaseRead::lend(&race.frame.lights_and_gear, scope),
            finish: PhaseRead::lend(&race.replay.finish, scope),
            // The frame's work and the panel and overlay.
            frame_work: PhaseRead::lend(&race.frame.frame_work, scope),
            panel_and_overlay: PhaseRead::lend(&race.frame.panel_and_overlay, scope),
            // The cars' blocks and the view variables.
            car_blocks: PhaseRead::lend(&race.replay.car_blocks, scope),
            view_and_records: PhaseRead::lend(&race.frame.view_and_records, scope),
            // The system's screen and the processor's MMX flag.
            screen: PhaseRead::lend(&system.session.screen, scope),
            has_mmx: has_mmx(system),
            // The course's groups.
            course_lists: PhaseRead::lend(&race.course.lists, scope),
            stage_prefs: PhaseRead::lend(&race.course.stage_prefs, scope),
            course_work: PhaseRead::lend(&race.course.course, scope),
            kind_records: PhaseRead::lend(race.course.kind_records_store(), scope),
            numbers: PhaseRead::lend(&race.course.numbers, scope),
            // The race owner's shared group (lane r41shared).
            switches: PhaseRead::lend(&race.shared.switches, scope),
            vars: PhaseRead::lend(&race.shared.vars, scope),
            view: PhaseRead::lend(&race.shared.view, scope),
            work: PhaseRead::lend(&race.shared.work, scope),
            physics_tables: PhaseRead::lend(&race.shared.physics_tables, scope),
            // The frame's matrices and the physics' contact work.
            matrices: PhaseRead::lend(&race.frame.matrices, scope),
            contact: PhaseRead::lend(&race.physics.contact, scope),
            // The race's buffers the parts read.
            buffers: PhaseRead::lend(&race.shared.buffers.reads, scope),
        }
    }

    /// The stores as the part on the calling thread reads them.
    fn reads(&self) -> DrawReads<'_> {
        DrawReads {
            light_tables: &self.light_tables,
            // The scene's other drawing stores.
            light_work: &self.light_work,
            draw_switches: &self.draw_switches,
            object_tables: &self.object_tables,
            object_slots: &self.object_slots,
            kind_textures: &self.kind_textures,
            texture_places: &self.texture_places,
            light_state: &self.light_state,
            // The scene's draw variables.
            scene_draw: &self.scene_draw,
            // The race's stores.
            level_grid: &self.level_grid,
            ground_probe: &self.ground_probe,
            shadow_skids: &self.shadow_skids,
            lights_and_gear: &self.lights_and_gear,
            finish: &self.finish,
            // The frame's work and the panel and overlay.
            frame_work: &self.frame_work,
            panel_and_overlay: &self.panel_and_overlay,
            // The cars' blocks and the view variables.
            car_blocks: &self.car_blocks,
            view_and_records: &self.view_and_records,
            // The system's screen and the processor's MMX flag.
            screen: &self.screen,
            has_mmx: self.has_mmx,
            // The course's groups.
            course_lists: &self.course_lists,
            stage_prefs: &self.stage_prefs,
            course_work: &self.course_work,
            kind_records: &self.kind_records,
            numbers: &self.numbers,
            // The race owner's shared group (lane r41shared).
            switches: &self.switches,
            vars: &self.vars,
            view: &self.view,
            work: &self.work,
            physics_tables: &self.physics_tables,
            // The frame's matrices and the physics' contact work.
            matrices: &self.matrices,
            contact: &self.contact,
            // The race's buffers the parts read.
            buffers: &self.buffers,
        }
    }
}

/// The class D stores as a drawing reads them: a part's from the scope's `Lent` (handed to
/// it by `draw_parts`), or the game's thread's from their owners outside a drawing scope
/// (`on_game_thread`). Read only: no drawing writes them.
#[derive(Clone, Copy)]
pub struct DrawReads<'a> {
    /// The lighting's colour tables (`scene::owner::Scene::light_tables`).
    pub light_tables: &'a crate::scene::light_tables_store::LightTables,
    // The scene's other drawing stores.
    /// The lights' work variables (`scene::owner::Lights::work`).
    pub light_work: &'a crate::scene::light_work_store::LightWork,
    /// The draw switches (`scene::owner::Scene::draw_switches`).
    pub draw_switches: &'a crate::scene::draw_switches_store::DrawSwitches,
    /// The object tables (`scene::owner::Objects::tables`).
    pub object_tables: &'a crate::scene::object_tables_store::ObjectTables,
    /// The object slots (`scene::owner::Objects::slots`).
    pub object_slots: &'a crate::scene::object_slots_store::ObjectTables,
    /// The kinds' texture tables (`scene::owner::Textures::kinds`).
    pub kind_textures: &'a crate::scene::kind_texture_store::KindTextureTables,
    /// The texture places (`scene::owner::Textures::places`).
    pub texture_places: &'a crate::scene::texture_places_store::TexturePlaces,
    /// The light state (`scene::owner::Lights::state`).
    pub light_state: &'a crate::scene::light_store::LightState,
    // The scene's draw variables.
    /// The scene's draw variables (`scene::owner::Drawing::scene`): the polygons' alpha, the
    /// fade and clip factors, the mirror picture's texture, the `v3driverfix` setting.
    pub scene_draw: &'a crate::scene::draw_store::SceneDraw,
    // The race's stores (`race::owner::Race`).
    /// The level grid's depth shift (`race::owner_course::Course::level_grid`).
    pub level_grid: &'a crate::race::level_grid_store::LevelGrid,
    /// The ground probe's mark (`race::owner_physics::Physics::ground_probe`).
    pub ground_probe: &'a crate::physics::ground_probe_store::GroundProbe,
    /// The skid marks' links and points (`race::owner_frame::Frame::shadow_skids`).
    pub shadow_skids: &'a crate::race::shadow_skids_store::ShadowSkids,
    /// The models' light records and the gear figures after them
    /// (`race::owner_frame::Frame::lights_and_gear`).
    pub lights_and_gear: &'a crate::race::groups_store::LightsAndGear,
    /// The finish routine's variables (`race::owner_replay::Replay::finish`).
    pub finish: &'a crate::drive::finish_store::Finish,
    // The frame's work and the panel and overlay.
    /// The frame's draw work (`race::owner_frame::Frame::frame_work`).
    pub frame_work: &'a crate::race::frame_store::FrameWork,
    /// The hud's panel and the race's overlay (`race::owner_frame::Frame::panel_and_overlay`).
    pub panel_and_overlay: &'a crate::race::groups_store::PanelAndOverlay,
    // The cars' blocks and the view variables.
    /// The cars' blocks (`race::owner_replay::Replay::car_blocks`).
    pub car_blocks: &'a crate::race::blocks_store::CarBlocks,
    /// The shared view variables and the view records
    /// (`race::owner_frame::Frame::view_and_records`).
    pub view_and_records: &'a crate::race::groups_store::ViewAndRecords,
    // The system's screen and the processor's MMX flag.
    /// The screen's figures (`system::owner::Session::screen`).
    pub screen: &'a crate::session::Screen,
    /// Whether the processor has MMX and the game uses it (`Machine::has_mmx`, set once at
    /// start-up before any drawing): the lighting's colour paths.
    pub has_mmx: bool,
    // The course's groups.
    /// The course's lists and work tables: the face-key factors, the root tables, the object
    /// points and point records (`race::owner_course::Course::lists`).
    pub course_lists: &'a crate::race::course_lists_store::CourseLists,
    /// The stage's prefs and scene settings: colours, fog, sun and sky, the face tables
    /// (`race::owner_course::Course::stage_prefs`).
    pub stage_prefs: &'a crate::race::prefs_store::StagePrefs,
    /// The course's data and the scene's state: the records, faces and scene points and their
    /// counts, the flat depth (`race::owner_course::Course::course`).
    pub course_work: &'a crate::race::course_store::CourseWork,
    /// The course kinds' first and last records (`race::owner_course::Course::kind_records`).
    pub kind_records: &'a crate::race::kind_records_store::KindRecords,
    /// The game's numbers: its float constants, the replay's mode, `in_race`, the grip change
    /// (`race::owner_course::Course::numbers`).
    pub numbers: &'a crate::race::numbers_store::Numbers,
    // The race owner's shared group (lane r41shared).
    /// The frame switches (`race::owner_shared::RaceShared::switches`): the scene set-up flag,
    /// the mask kind, the marker object.
    pub switches: &'a crate::race::switches_store::FrameSwitches,
    /// The race's variables (`race::owner_shared::RaceShared::vars`): the settings the drawing
    /// reads (run kind, split, the settings words), the mud texture's state, the fade figures.
    pub vars: &'a crate::race::vars_store::RaceVars,
    /// The race's view and face work (`race::owner_shared::RaceShared::view`): the players
    /// and views, their cars, the face tables.
    pub view: &'a crate::race::view_store::RaceView,
    /// The race's work variables (`race::owner_shared::RaceShared::work`): the cars' block
    /// handles, the number textures, the loose parts' slots, the credits flag.
    pub work: &'a crate::race::work_store::RaceWork,
    /// The physics' tables and work variables (`race::owner_shared::RaceShared::physics_tables`):
    /// the ground kinds, the scene's clock, the drops the models are placed by.
    pub physics_tables: &'a crate::physics::tables_store::PhysicsTables,
    // The frame's matrices and the physics' contact work.
    /// The frame's matrices (`race::owner_frame::Frame::matrices`): the identity and the work
    /// matrices the views are made from, the fourth columns the views hold from.
    pub matrices: &'a crate::race::matrices_store::SceneMatrices,
    /// The physics' contact work (`race::owner_physics::Physics::contact`): the lit faces the
    /// lamps are gathered from.
    pub contact: &'a crate::physics::contact_store::ContactWork,
    // The race's buffers the parts read.
    /// The polygon tables (the textures' records, the objects' materials), the faces, the
    /// checkpoints and the engine tables (`race::buffers_store::Buffers::reads`).
    pub buffers: &'a crate::race::buffers_store::BufferReads,
}

impl<'a> DrawReads<'a> {
    // r40course
    /// The course's drawing stores as one value (`race::owner_course::CourseReads`).
    pub fn course(&self) -> crate::race::owner_course::CourseReads<'a> {
        crate::race::owner_course::CourseReads {
            course: self.course_work,
            lists: self.course_lists,
            stage_prefs: self.stage_prefs,
            numbers: self.numbers,
            kind_records: self.kind_records,
        }
    }
    // end r40course

    /// The stores from their owners: on the game's thread, outside a drawing scope (the
    /// views drawn one after another, the overlays, the menus' objects).
    pub fn on_game_thread(owners: LentOwners<'a>) -> Self {
        let LentOwners {
            race,
            scene,
            system,
        } = owners;
        Self {
            light_tables: scene.light_tables.get(),
            // The scene's other drawing stores.
            light_work: scene.lights.work.get(),
            draw_switches: scene.draw_switches.get(),
            object_tables: scene.objects.tables.get(),
            object_slots: scene.objects.slots.get(),
            kind_textures: scene.textures.kinds.get(),
            texture_places: scene.textures.places.get(),
            light_state: scene.lights.state.get(),
            // The scene's draw variables.
            scene_draw: scene.drawing.scene.get(),
            // The race's stores.
            level_grid: race.course.level_grid.get(),
            ground_probe: race.physics.ground_probe.get(),
            shadow_skids: race.frame.shadow_skids.get(),
            lights_and_gear: race.frame.lights_and_gear.get(),
            finish: race.replay.finish.get(),
            // The frame's work and the panel and overlay.
            frame_work: race.frame.frame_work.get(),
            panel_and_overlay: race.frame.panel_and_overlay.get(),
            // The cars' blocks and the view variables.
            car_blocks: race.replay.car_blocks.get(),
            view_and_records: race.frame.view_and_records.get(),
            // The system's screen and the processor's MMX flag.
            screen: system.session.screen.get(),
            has_mmx: has_mmx(system),
            // The course's groups.
            course_lists: race.course.lists.get(),
            stage_prefs: race.course.stage_prefs.get(),
            course_work: race.course.course.get(),
            kind_records: race.course.kind_records(),
            numbers: race.course.numbers.get(),
            // The race owner's shared group (lane r41shared).
            switches: race.shared.switches.get(),
            vars: race.shared.vars.get(),
            view: race.shared.view.get(),
            work: race.shared.work.get(),
            physics_tables: race.shared.physics_tables.get(),
            // The frame's matrices and the physics' contact work.
            matrices: race.frame.matrices.get(),
            contact: race.physics.contact.get(),
            // The race's buffers the parts read.
            buffers: race.shared.buffers.reads.get(),
        }
    }
}

// ---- the runs ----

/// The rear-view mirror drawn beside its view (`draw_beside`, `race::draw_race_scene`): on a
/// machine of two cores or more, drawing and sounding the same as one after the other.
/// `RC2K_MIRROR_BESIDE=0` turns it off, any other value on.
pub fn mirror_beside() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| match std::env::var_os("RC2K_MIRROR_BESIDE") {
        Some(v) => v != "0",
        None => std::thread::available_parallelism().map_or(1, usize::from) >= 2,
    })
}

/// Whether parts are being drawn beside each other (`draw_beside`): a list inside one of them
/// is not split into runs, the copies being the parts' own.
static BESIDE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether parts are being drawn beside each other (`draw_beside`).
pub fn beside() -> bool {
    BESIDE.load(std::sync::atomic::Ordering::Relaxed)
}

/// With `RC2K_DRAW_SERIAL` (a check): the runs drawn one after another on the game's thread,
/// each on its own copy as on its own thread, so that what the copies and their merging do is
/// told apart from what threads drawing at once do.
fn serial() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("RC2K_DRAW_SERIAL").is_some())
}

/// The other drawing threads: with `mirror_beside`, one fewer than the cores, up to the
/// copies (the mirror beside its view uses one: the handheld at 25 to 30 frames a second; the
/// players' views and mirrors beside each other up to three).
fn pool() -> Option<&'static rayon::ThreadPool> {
    static POOL: OnceLock<Option<rayon::ThreadPool>> = OnceLock::new();
    POOL.get_or_init(|| {
        let cores = std::thread::available_parallelism().map_or(1, usize::from);
        let all = if mirror_beside() { cores.min(COUNT) } else { 1 };
        (all > 1)
            .then(|| {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(all - 1)
                    .thread_name(|n| format!("draw {}", n + 1))
                    .build()
                    .ok()
            })
            .flatten()
    })
    .as_ref()
}

/// The runs of the list being drawn that have ended (by run number).
static ENDED: Mutex<[bool; COUNT]> = Mutex::new([false; COUNT]);
static ENDED_CHANGED: Condvar = Condvar::new();

fn ended(n: usize) {
    let mut e = ENDED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    e[n] = true;
    ENDED_CHANGED.notify_all();
}

/// Waits until the runs before this thread's have ended: what must be done in the list's
/// order (the random numbers, `math::next_random`) is then done after them, as by one
/// thread. Nothing to wait for on the game's thread (run 0) or outside a list.
pub fn wait_for_earlier_runs() {
    let n = index();
    if n == 0 {
        return;
    }
    if n == ASIDE_INDEX {
        // A piece drawn aside (`draw_aside`): after what its part draws before it.
        wait_for_aside_turn();
        return;
    }
    let mut e = ENDED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    while e[..n].iter().any(|&done| !done) {
        e = ENDED_CHANGED
            .wait(e)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
    }
}

/// `first` drawn on the game's thread and `second` beside it on a drawing thread, each from
/// the working state as it is now (the mirror beside its view, `race::draw_race_scene`), then
/// `then` on the game's thread: `then`'s drawing goes after `first`'s, which may flush, and
/// `second`'s after `then`'s, its changes to the working state over theirs; then
/// `then_second`. With fewer than two drawing threads both on the game's thread, one after
/// the other, each on its own copy as on its own thread, so that what is drawn is the same
/// however many threads draw.
pub fn draw_beside(
    owners: LentOwners<'_>,
    slots: [usize; 2],
    first: impl Fn(PartRun<'_>, DrawReads<'_>) -> Result<(), Refused> + Sync,
    second: impl Fn(PartRun<'_>, DrawReads<'_>) -> Result<(), Refused> + Sync,
    mut then: impl FnMut() -> Result<(), Refused>,
    mut then_second: impl FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    draw_side_by_side(
        owners,
        &slots,
        |run, reads| {
            if run.n == 0 {
                first(run, reads)
            } else {
                second(run, reads)
            }
        },
        |n| if n == 0 { then() } else { then_second() },
    )
}

/// What a part's run draws on: its number, its own scratch and the view slot it draws
/// (`race::view_scratch::ViewScratch`), claimed for the run by `draw_parts`.
#[derive(Clone, Copy)]
pub struct PartRun<'a> {
    pub n: usize,
    pub part: &'a PartScratch,
    pub view: &'a ViewScratch,
    /// What the part may hand a piece of its drawing to (`draw_aside`): part 0's only, when
    /// the pool has a thread for it besides the other parts' (`Helper::for_first_part`).
    pub helper: Option<Helper<'a>>,
}

/// Parts 0 .. `slots.len()` (`part(run, reads)`, at most `COUNT`, each on its own scratch and
/// the view slot `slots[n]`, which are distinct, reading
/// the class D stores of `owners` through `reads`) drawn beside
/// each other as `draw_beside` draws two: the first on the game's thread, each other on a drawing thread, each from the working
/// state as it is now; their drawing put together and their changes put back in their order.
/// `after(n)` runs on the game's thread once part `n` is put back (part 0: once every part
/// has ended), before the next is: what the drawing does there goes after part `n`'s, on the
/// working state as the parts so far left it, and no part runs meanwhile (what it writes that
/// has no copy per part, the cars' crumpled points, no part reads while it changes).
pub fn draw_side_by_side(
    owners: LentOwners<'_>,
    slots: &[usize],
    part: impl Fn(PartRun<'_>, DrawReads<'_>) -> Result<(), Refused> + Sync,
    after: impl FnMut(usize) -> Result<(), Refused>,
) -> Result<(), Refused> {
    if slots.len() > COUNT {
        return Err(BadInput("more parts beside each other than drawing copies").into());
    }
    // Each part draws a view slot of its own: the views are claimed one to a run.
    if !crate::race::view_scratch::drawable_slots(slots) {
        return Err(BadInput(
            "two parts of one drawing on one view slot, or a slot not drawn by parts",
        )
        .into());
    }
    let pool = pool();
    BESIDE.store(true, std::sync::atomic::Ordering::Relaxed);
    let drawn = draw_parts(owners, pool, slots, part, after);
    BESIDE.store(false, std::sync::atomic::Ordering::Relaxed);
    drawn
}

/// The two players' views and their mirrors drawn beside each other in split screen
/// (`draw_side_by_side`, `race::draw_race_scene`): with the mirror beside its view
/// (`mirror_beside`). `RC2K_PLAYERS_BESIDE=0` turns it off, any other value on.
pub fn players_beside() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| match std::env::var_os("RC2K_PLAYERS_BESIDE") {
        Some(v) => v != "0",
        None => mirror_beside(),
    })
}

/// Parts 0 .. `slots.len()` of a drawing (`part`), part 0 on the game's thread and each other on a
/// drawing thread with its own copy of the working state, all set from the game's own at the
/// start; the other parts' noted drawing put after part 0's in their order, and each copy's
/// changes put into the game's own in that order, `after(n)` run after part `n`'s (part 0's
/// once all have ended). While the parts are drawn (a `DrawScope`) the class D stores are lent
/// to them (`Lent`): each part reads them through the `DrawReads` it is handed, which lives
/// only for its call.
fn draw_parts(
    owners: LentOwners<'_>,
    pool: Option<&rayon::ThreadPool>,
    slots: &[usize],
    part: impl Fn(PartRun<'_>, DrawReads<'_>) -> Result<(), Refused> + Sync,
    mut after: impl FnMut(usize) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let k = slots.len();
    // One after another on the game's thread without a pool or with `RC2K_DRAW_SERIAL`.
    let pool = pool.filter(|_| !serial());
    // The render states' cache as the list began, what each other part starts from.
    let first_cache = &part_scratch(0).cache;
    let cache_start = crate::d3d::render_cache::current(first_cache);
    // The held matrix columns likewise (`scene::view_matrices::Held`), in the race's
    // matrices; every part, the first too, holds its own while the scope is open.
    let point_count = owners.race.course.course.get().scene_point_count.get();
    let matrices = &owners.race.frame.matrices;
    let columns_start = crate::scene::view_matrices::columns(matrices.get());
    *ENDED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = [false; COUNT];
    let control = crate::math::x87::control_word();
    // r40course
    // Whether a race is on, as the parts' noted drawing reads it inside the scope (`in_race`).
    let racing = owners.race.course.numbers.get().in_race.is_set();
    let screen = owners.system.session.screen.get();
    let screen_size = [screen.width.get(), screen.height.get(), screen.bits.get()];
    for scratch in &PARTS {
        let scratch = scratch.on_game_thread();
        scratch.in_race.set(racing);
        for (cell, figure) in scratch.screen_size.iter().zip(screen_size) {
            cell.set(figure);
        }
    }
    // end r40course
    // The parts are drawn from here until every one has ended: the class D stores lent.
    let scope = DrawScope::open();
    let lent = Lent::new(&scope, owners);
    let lent = &lent;
    let scope_ref = &scope;
    // Run `n` of the others, on the thread it is called on, with its own noted drawing.
    let other = |n: usize| {
        INDEX.set(n);
        let turn = PartTurn::new(scope_ref, n);
        let scratch = PARTS[n].claim(&turn);
        let view = crate::race::view_scratch::claim(slots[n], &turn);
        scratch.points.begin_run(point_count);
        crate::math::x87::set_control_word(control);
        crate::d3d::run_states::begin_run(scratch, &cache_start);
        crate::scene::view_matrices::begin_part(scratch, columns_start);
        let result = part(
            PartRun {
                n,
                part: scratch,
                view,
                helper: None,
            },
            lent.reads(),
        );
        let states = crate::d3d::run_states::end_run(scratch);
        let columns = crate::scene::view_matrices::end_part(scratch).unwrap_or(columns_start);
        ended(n);
        scratch.points.end_run();
        let noted = crate::platform_3d::take_noted(scratch);
        INDEX.set(0);
        OtherRun {
            result,
            noted,
            states,
            columns,
        }
    };
    // Run 0, on the game's thread.
    let first_run = || {
        let turn = PartTurn::new(scope_ref, 0);
        let scratch = PARTS[0].claim(&turn);
        let view = crate::race::view_scratch::claim(slots[0], &turn);
        scratch.points.begin_run(point_count);
        crate::scene::view_matrices::begin_part(scratch, columns_start);
        let result = part(
            PartRun {
                n: 0,
                part: scratch,
                view,
                helper: Helper::for_first_part(pool, k, lent),
            },
            lent.reads(),
        );
        let columns = crate::scene::view_matrices::end_part(scratch).unwrap_or(columns_start);
        scratch.points.end_run();
        ended(0);
        (result, columns)
    };
    let (first, others) = if let Some(pool) = pool {
        let slots: Vec<Mutex<Option<OtherRun>>> = (1..k).map(|_| Mutex::new(None)).collect();
        let mut first = (Ok(()), columns_start);
        // In place: the closure (run 0) on the game's thread, the spawned runs on the pool.
        pool.in_place_scope(|s| {
            for (n, slot) in slots.iter().enumerate() {
                let other = &other;
                s.spawn(move |_| {
                    let run = other(n + 1);
                    *slot
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(run);
                });
            }
            first = first_run();
        });
        let others = slots
            .into_iter()
            .map(|slot| {
                slot.into_inner()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .ok_or(BadInput("a drawing run that did not end"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        (first, others)
    } else {
        let first = first_run();
        let others: Vec<OtherRun> = (1..k).map(other).collect();
        (first, others)
    };
    // Every part has ended: the stores are the game's thread's again.
    drop(scope);
    let (first, first_columns) = first;
    // Part 0's held columns go into the store before anything runs on the game's thread.
    crate::scene::view_matrices::merge_columns(matrices.get(), &columns_start, &first_columns);
    let mut result = first;
    // Part 0 is put back: what follows it draws on the game's thread, every part ended.
    if result.is_ok() {
        result = after(0);
    }
    for (n, mut run) in others.into_iter().enumerate() {
        result = result.and(run.result);
        // Its fans before it set a state draw with what the earlier runs left.
        let carried = crate::d3d::run_states::carried(first_cache);
        crate::d3d::run_states::patch(&mut run.noted.commands, &run.states, &carried);
        crate::platform_3d::put_noted(run.noted);
        crate::d3d::run_states::carry(first_cache, &run.states, &cache_start);
        crate::scene::view_matrices::merge_columns(matrices.get(), &columns_start, &run.columns);
        if result.is_ok() {
            // What it draws takes the merged states.
            crate::platform_3d::forget_states(part_scratch(0));
            result = after(n + 1);
        }
    }
    // The game's thread drew run 0 with its own states: the next fan takes the merged ones.
    crate::platform_3d::forget_states(part_scratch(0));
    result
}

/// What a drawing thread's run left.
struct OtherRun {
    result: Result<(), Refused>,
    noted: crate::render::CommandList,
    /// Where it set the render states.
    states: crate::d3d::run_states::RunStates,
    /// The held matrix columns it left.
    columns: crate::scene::view_matrices::Columns,
}

// ---- a piece of a part drawn aside ----

/// Whether the player's car is drawn as a piece of its own beside the rest of its view
/// (`race::car_part`, `draw_aside`): `RC2K_CAR_PART=0` off (the view drawn on its part's
/// thread alone, as before, for a comparison of times), any other value or none on.
pub fn car_part() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("RC2K_CAR_PART").as_deref() != Ok("0"))
}

/// Whether a view drawn alone is drawn as a part of one (`draw_side_by_side`) for its car to
/// be drawn aside: the car part on and a drawing thread for it, or the parts drawn one after
/// another (`RC2K_DRAW_SERIAL`).
pub fn car_part_wanted() -> bool {
    car_part() && (pool().is_some() || serial())
}

/// What a part may hand a piece of its own drawing to (`draw_aside`): the drawing threads (none
/// with `RC2K_DRAW_SERIAL`: the piece is then drawn on the part's own thread when it is joined,
/// on its copies all the same) and the stores lent to the scope.
#[derive(Clone, Copy)]
pub struct Helper<'a> {
    pool: Option<&'a rayon::ThreadPool>,
    lent: &'a Lent<'a>,
}

impl<'a> Helper<'a> {
    /// Part 0's helper in a scope of `parts` parts drawn on `pool` (none: one after another):
    /// with the car part on, when the pool has a thread to spare besides the other parts' (each
    /// part waits for nothing a later one does, so the piece never waits for a thread).
    fn for_first_part(
        pool: Option<&'a rayon::ThreadPool>,
        parts: usize,
        lent: &'a Lent<'a>,
    ) -> Option<Self> {
        if !car_part() {
            return None;
        }
        match pool {
            Some(p) if p.current_num_threads() >= parts => Some(Self {
                pool: Some(p),
                lent,
            }),
            None if serial() => Some(Self { pool: None, lent }),
            _ => None,
        }
    }
}

/// The run number of a piece drawn aside (`INDEX` on its thread): past the parts'.
const ASIDE_INDEX: usize = COUNT;

/// The scratch a piece drawn aside draws on: one piece at a time, its thread holding the lock
/// for its run (`draw_aside`), so it is reached by one thread at a time with no `unsafe`.
static ASIDE: Mutex<PartScratch> = Mutex::new(PartScratch::new(false));

/// Whether the part drawing a piece aside has come to the piece's place in its drawing
/// (`Aside::join`): what the piece must do in the drawing's order (`wait_for_earlier_runs`)
/// waits for it.
static ASIDE_TURN: Mutex<bool> = Mutex::new(false);
static ASIDE_TURN_CHANGED: Condvar = Condvar::new();

fn set_aside_turn(reached: bool) {
    *ASIDE_TURN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = reached;
    ASIDE_TURN_CHANGED.notify_all();
}

fn wait_for_aside_turn() {
    let mut reached = ASIDE_TURN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    while !*reached {
        reached = ASIDE_TURN_CHANGED
            .wait(reached)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
    }
}

/// What a part's state was as it handed a piece aside: what the piece starts from.
struct AsideStart {
    point_count: u32,
    cache: crate::d3d::render_cache::Cache,
    columns: Option<crate::scene::view_matrices::Columns>,
    control: u16,
    in_race: bool,
    screen_size: [u32; 3],
}

/// What a piece drawn aside left: its result, its noted drawing (its fans before its own
/// setting of a state given the part's value of it at the piece's place, `Aside::join`) and
/// where it set the render states and the texture.
pub struct AsideRun {
    pub result: Result<(), Refused>,
    pub noted: crate::render::CommandList,
    states: crate::d3d::run_states::RunStates,
}

/// The piece drawn on `scratch` (on the thread that holds it) from `start`.
fn run_aside(
    scratch: &PartScratch,
    start: &AsideStart,
    reads: DrawReads<'_>,
    piece: impl FnOnce(&PartScratch, DrawReads<'_>) -> Result<(), Refused>,
) -> AsideRun {
    scratch.points.begin_run(start.point_count);
    scratch.in_race.set(start.in_race);
    for (cell, figure) in scratch.screen_size.iter().zip(start.screen_size) {
        cell.set(figure);
    }
    crate::math::x87::set_control_word(start.control);
    crate::d3d::run_states::begin_run(scratch, &start.cache);
    crate::scene::view_matrices::set_part_columns(scratch, start.columns);
    let result = piece(scratch, reads);
    let states = crate::d3d::run_states::end_run(scratch);
    crate::scene::view_matrices::end_part(scratch);
    scratch.points.end_run();
    let noted = crate::platform_3d::take_noted(scratch);
    AsideRun {
        result,
        noted,
        states,
    }
}

/// A piece of a part's drawing: what it draws, on its own scratch, reading the scope's stores.
pub type Piece<'a> =
    Box<dyn FnOnce(&PartScratch, DrawReads<'_>) -> Result<(), Refused> + Send + 'a>;

/// A piece of a part's drawing being drawn aside (`draw_aside`), handed to the rest of the
/// part's drawing: `join` where the piece goes in the drawing's order, `put` its drawing
/// there.
pub struct Aside<'a> {
    part: &'a PartScratch,
    start: AsideStart,
    /// The piece's run, sent by its thread.
    ran: Option<std::sync::mpsc::Receiver<AsideRun>>,
    /// The piece itself while it is to be drawn when joined (no drawing threads).
    later: std::cell::RefCell<Option<(Piece<'a>, &'a Lent<'a>)>>,
    joined: Cell<bool>,
}

impl Aside<'_> {
    /// The piece's run, once it has ended (drawn here now without drawing threads), its fans
    /// before its own setting of a state given the part's value of it now. Once only.
    pub fn join(&self) -> Result<AsideRun, Refused> {
        if self.joined.replace(true) {
            return Err(BadInput("a piece drawn aside joined twice").into());
        }
        set_aside_turn(true);
        let mut run = match &self.ran {
            Some(ran) => ran
                .recv()
                .map_err(|_| BadInput("a piece drawn aside that did not end"))?,
            None => {
                let (piece, lent) = self
                    .later
                    .borrow_mut()
                    .take()
                    .ok_or(BadInput("a piece drawn aside with nothing to draw"))?;
                let scratch = ASIDE
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                run_aside(&scratch, &self.start, lent.reads(), piece)
            }
        };
        let carried = crate::d3d::run_states::carried(&self.part.cache);
        crate::d3d::run_states::patch(&mut run.noted.commands, &run.states, &carried);
        let now = crate::d3d::render_cache::current(&self.part.cache);
        if crate::d3d::run_states::drew_before_texture(&run.noted.commands, &run.states)
            && !crate::d3d::render_cache::same_texture(&now, &self.start.cache)
        {
            crate::framework::log(
                "workers: a piece drawn aside drew before setting its texture (not as one thread)",
            );
        }
        Ok(run)
    }

    /// `run`'s drawing put after the part's so far, and the render states and the texture it
    /// set into the part's cache.
    pub fn put(&self, run: AsideRun) -> Result<(), Refused> {
        debug_assert!(
            std::ptr::eq(self.part, part_scratch(0)),
            "a piece drawn aside of a part other than the first"
        );
        crate::platform_3d::put_noted(run.noted);
        crate::d3d::run_states::carry_set(&self.part.cache, &run.states);
        // The point records it wrote, as the part would have them after it; the piece's own
        // set back to 0 for the next piece.
        {
            let scratch = ASIDE
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            crate::scene::point_records::put_written(&scratch, self.part);
            crate::scene::point_records::clear_written(&scratch);
        }
        crate::platform_3d::forget_states(self.part);
        run.result
    }

    /// The part's drawing is over: a piece not joined is waited for and its drawing dropped
    /// (it had no place in the drawing).
    fn finish(&self) {
        if self.joined.replace(true) {
            return;
        }
        set_aside_turn(true);
        if let Some(ran) = &self.ran {
            let _ = ran.recv();
            crate::scene::point_records::clear_written(
                &ASIDE
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );
        }
    }
}

/// The piece's turn given when dropped: a part's drawing that ends early (a refusal, a panic)
/// never leaves its piece waiting for a turn that will not come.
struct GiveTurn;

impl Drop for GiveTurn {
    fn drop(&mut self) {
        set_aside_turn(true);
    }
}

/// `piece` drawn aside of part `part` (the first, on the game's thread) on a drawing thread of
/// `helper`'s, on a scratch of its own set from the part's as it is now (its render states,
/// its held matrix columns, its scene points past `point_count`; its point records all 0, the
/// ones it writes put into the part's when it is put, so a piece must make every record it
/// reads: the view's car's held races drew the same), while `rest` draws on in the part;
/// `rest` gets the piece as an `Aside`, to join and put where it goes in the drawing's order.
/// What the piece must do in that order (its random numbers) waits until it is joined.
/// Without drawing threads the piece is drawn when it is joined, on the part's thread.
pub fn draw_aside<'a, R>(
    helper: Helper<'a>,
    part: &'a PartScratch,
    point_count: u32,
    piece: Piece<'a>,
    rest: impl FnOnce(&Aside<'a>) -> R,
) -> R {
    let start = AsideStart {
        point_count,
        cache: crate::d3d::render_cache::current(&part.cache),
        columns: crate::scene::view_matrices::part_columns(part),
        control: crate::math::x87::control_word(),
        in_race: part.in_race.is_set(),
        screen_size: part
            .screen_size
            .each_ref()
            .map(crate::session::PlainDword::get),
    };
    set_aside_turn(false);
    let Some(pool) = helper.pool else {
        let aside = Aside {
            part,
            start,
            ran: None,
            later: std::cell::RefCell::new(Some((piece, helper.lent))),
            joined: Cell::new(false),
        };
        let r = rest(&aside);
        // Not joined: never drawn.
        aside.joined.set(true);
        return r;
    };
    let (send, ran) = std::sync::mpsc::sync_channel(1);
    let aside = Aside {
        part,
        start,
        ran: Some(ran),
        later: std::cell::RefCell::new(None),
        joined: Cell::new(false),
    };
    let lent = helper.lent;
    let begin = &aside.start;
    pool.in_place_scope(|s| {
        s.spawn(move |_| {
            INDEX.set(ASIDE_INDEX);
            let run = {
                let scratch = ASIDE
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                run_aside(&scratch, begin, lent.reads(), piece)
            };
            INDEX.set(0);
            let _ = send.send(run);
        });
        let _turn = GiveTurn;
        let r = rest(&aside);
        aside.finish();
        r
    })
}
