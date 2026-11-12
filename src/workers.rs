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

use crate::errors::{BadInput, Refused};
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
    pub vertices: &'static crate::d3d::vertices_store::PolygonVertices,
    /// The polygon work: the entries, their lists, the clipper's pool
    /// (`scene::polygon_store`).
    pub polygon: &'static crate::scene::polygon_store::PolygonWork,
    /// The point records, a cache of the points' projections (`scene::point_records`).
    pub records: &'static crate::scene::point_records::Records,
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
    /// thread flushes, the others' put after it at their runs' ends.
    pub noted: crate::platform_3d::Noted,
    /// Whether a race is on (`race::numbers_store::Numbers::in_race`) as the parts of a
    /// drawing scope read it, set from the store before the scope opens (`draw_parts`): the
    /// GPU draws only then. Read through `in_race`.
    pub in_race: crate::session::Flag,
}

/// The parts' scratch, part 0's in the stores' places in the race's buffers (the image's
/// layout) and the game's own records, the others' their stores' other copies.
static PARTS: [PartScratch; COUNT] = {
    use crate::d3d::vertices_store::OTHER_COPIES as V;
    use crate::scene::point_records::RECORDS as R;
    use crate::scene::polygon_store::OTHER_COPIES as P;
    let buffers = &crate::race::owner::RACE.shared.buffers;
    [
        PartScratch {
            vertices: &buffers.vertices,
            polygon: &buffers.polygon,
            records: &R[0],
            columns: crate::scene::view_matrices::OwnColumns::new(),
            points: crate::race::scene_points::OwnPoints::new(),
            cache: crate::d3d::render_cache::OwnCache::first(),
            run: crate::d3d::run_states::OwnRun::new(),
            noted: crate::platform_3d::Noted::new(),
            in_race: crate::session::Flag::new(false),
        },
        PartScratch {
            vertices: &V[0],
            polygon: &P[0],
            records: &R[1],
            columns: crate::scene::view_matrices::OwnColumns::new(),
            points: crate::race::scene_points::OwnPoints::new(),
            cache: crate::d3d::render_cache::OwnCache::new(),
            run: crate::d3d::run_states::OwnRun::new(),
            noted: crate::platform_3d::Noted::new(),
            in_race: crate::session::Flag::new(false),
        },
        PartScratch {
            vertices: &V[1],
            polygon: &P[1],
            records: &R[2],
            columns: crate::scene::view_matrices::OwnColumns::new(),
            points: crate::race::scene_points::OwnPoints::new(),
            cache: crate::d3d::render_cache::OwnCache::new(),
            run: crate::d3d::run_states::OwnRun::new(),
            noted: crate::platform_3d::Noted::new(),
            in_race: crate::session::Flag::new(false),
        },
        PartScratch {
            vertices: &V[2],
            polygon: &P[2],
            records: &R[3],
            columns: crate::scene::view_matrices::OwnColumns::new(),
            points: crate::race::scene_points::OwnPoints::new(),
            cache: crate::d3d::render_cache::OwnCache::new(),
            run: crate::d3d::run_states::OwnRun::new(),
            noted: crate::platform_3d::Noted::new(),
            in_race: crate::session::Flag::new(false),
        },
    ]
};

/// Part `n`'s scratch (below `COUNT`).
pub fn part_scratch(n: usize) -> &'static PartScratch {
    &PARTS[n]
}

/// The game's own thread's scratch: part 0's, what everything drawn outside the parts (the
/// overlays, the HUD, the front end) works on.
pub fn game_scratch() -> &'static PartScratch {
    debug_assert_eq!(index(), 0, "the game's scratch on a drawing part");
    &PARTS[0]
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
        crate::race::owner::RACE
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
}

/// Whether the processor has MMX and the game uses it: fixed at start-up, so one value for
/// every part of a drawing.
fn has_mmx(system: &crate::system::owner::System) -> bool {
    system.session.machine.has_mmx.get() != 0
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
    matrices: &crate::race::matrices_store::SceneMatrices,
    owners: LentOwners<'_>,
    first: impl Fn(&PartScratch, DrawReads<'_>) -> Result<(), Refused> + Sync,
    second: impl Fn(&PartScratch, DrawReads<'_>) -> Result<(), Refused> + Sync,
    mut then: impl FnMut() -> Result<(), Refused>,
    mut then_second: impl FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    draw_side_by_side(
        matrices,
        owners,
        2,
        |n, scratch, reads| {
            if n == 0 {
                first(scratch, reads)
            } else {
                second(scratch, reads)
            }
        },
        |n| if n == 0 { then() } else { then_second() },
    )
}

/// Parts 0 .. `k` (`part(n, scratch, reads)`, at most `COUNT`, each on its own scratch, reading
/// the class D stores of `owners` through `reads`) drawn beside
/// each other as `draw_beside` draws two: the first on the game's thread, each other on a drawing thread, each from the working
/// state as it is now; their drawing put together and their changes put back in their order.
/// `after(n)` runs on the game's thread once part `n` is put back (part 0: once every part
/// has ended), before the next is: what the drawing does there goes after part `n`'s, on the
/// working state as the parts so far left it, and no part runs meanwhile (what it writes that
/// has no copy per part, the cars' crumpled points, no part reads while it changes).
pub fn draw_side_by_side(
    matrices: &crate::race::matrices_store::SceneMatrices,
    owners: LentOwners<'_>,
    k: usize,
    part: impl Fn(usize, &PartScratch, DrawReads<'_>) -> Result<(), Refused> + Sync,
    after: impl FnMut(usize) -> Result<(), Refused>,
) -> Result<(), Refused> {
    if k > COUNT {
        return Err(BadInput("more parts beside each other than drawing copies").into());
    }
    let pool = pool();
    BESIDE.store(true, std::sync::atomic::Ordering::Relaxed);
    let drawn = draw_parts(matrices, owners, pool, k, part, after);
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

/// Parts 0 .. `k` of a drawing (`part`), part 0 on the game's thread and each other on a
/// drawing thread with its own copy of the working state, all set from the game's own at the
/// start; the other parts' noted drawing put after part 0's in their order, and each copy's
/// changes put into the game's own in that order, `after(n)` run after part `n`'s (part 0's
/// once all have ended). While the parts are drawn (a `DrawScope`) the class D stores are lent
/// to them (`Lent`): each part reads them through the `DrawReads` it is handed, which lives
/// only for its call.
fn draw_parts(
    matrices: &crate::race::matrices_store::SceneMatrices,
    owners: LentOwners<'_>,
    pool: Option<&rayon::ThreadPool>,
    k: usize,
    part: impl Fn(usize, &PartScratch, DrawReads<'_>) -> Result<(), Refused> + Sync,
    mut after: impl FnMut(usize) -> Result<(), Refused>,
) -> Result<(), Refused> {
    // One after another on the game's thread without a pool or with `RC2K_DRAW_SERIAL`.
    let pool = pool.filter(|_| !serial());
    // The render states' cache as the list began, what each other part starts from.
    let first_cache = &part_scratch(0).cache;
    let cache_start = crate::d3d::render_cache::current(first_cache);
    // The held matrix columns likewise (`scene::view_matrices::Held`), in the race's
    // matrices `matrices`.
    let point_count = owners.race.course.course.get().scene_point_count.get();
    let columns_start = crate::scene::view_matrices::columns(matrices);
    *ENDED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = [false; COUNT];
    let control = crate::math::x87::control_word();
    // r40course
    // Whether a race is on, as the parts' noted drawing reads it inside the scope (`in_race`).
    let racing = owners.race.course.numbers.get().in_race.is_set();
    for scratch in &PARTS {
        scratch.in_race.set(racing);
    }
    // end r40course
    // The parts are drawn from here until every one has ended: the class D stores lent.
    let scope = DrawScope::open();
    let lent = Lent::new(&scope, owners);
    let lent = &lent;
    // Run `n` of the others, on the thread it is called on, with its own noted drawing.
    let other = |n: usize| {
        INDEX.set(n);
        let scratch = part_scratch(n);
        scratch.points.begin_run(point_count);
        crate::math::x87::set_control_word(control);
        crate::d3d::run_states::begin_run(scratch, &cache_start);
        crate::scene::view_matrices::begin_part(scratch, columns_start);
        let result = part(n, scratch, lent.reads());
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
        let scratch = part_scratch(0);
        scratch.points.begin_run(point_count);
        let result = part(0, scratch, lent.reads());
        scratch.points.end_run();
        ended(0);
        result
    };
    let (first, others) = if let Some(pool) = pool {
        let slots: Vec<Mutex<Option<OtherRun>>> = (1..k).map(|_| Mutex::new(None)).collect();
        let mut first = Ok(());
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
    let mut result = first;
    // Part 0 is put back: what follows it draws on the game's thread, every part ended.
    if result.is_ok() {
        result = after(0);
    }
    for (n, mut run) in others.into_iter().enumerate() {
        result = result.and(run.result);
        // Its fans before it set a state draw with what the earlier runs left.
        let carried = crate::d3d::run_states::carried(first_cache);
        crate::d3d::run_states::patch(&mut run.noted, &run.states, &carried);
        crate::platform_3d::put_noted(run.noted);
        crate::d3d::run_states::carry(first_cache, &run.states, &cache_start);
        crate::scene::view_matrices::merge_columns(matrices, &columns_start, &run.columns);
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
    noted: Vec<crate::render::Command>,
    /// Where it set the render states.
    states: crate::d3d::run_states::RunStates,
    /// The held matrix columns it left.
    columns: crate::scene::view_matrices::Columns,
}
