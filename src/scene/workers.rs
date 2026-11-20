//! The drawing's threads. Parts of a frame (the players' views and their mirrors) drawn by several
//! threads at once, so the frame is what one thread draws. Each view draws on its own working state
//! (`race::view_scratch`); what a view hands on to the shared stores is applied on the game's
//! thread in the views' order (`race::draw_race_scene`'s `view_drawn`), and the render states'
//! cache and the held matrix columns a part keeps are put back in the parts' order (frame plan F7:
//! no drawing part writes a shared store).
//!
//! The scratch stores each part makes for itself before reading (the polygon vertex buffer,
//! the polygon work, the point records) are a `PartScratch`: one per part (`DrawingScratch`),
//! handed to the part's drawing and passed down to what reaches them. Part 0's is the game's
//! own thread's, what everything drawn outside the parts works on too
//! (`DrawingScratch::game_scratch`).
//! A `PartScratch` is made of plain cells (and the `RefCell` of its noted drawing, which only
//! the thread that has claimed the part reaches): it is `Send`, asserted below.
//!
//! Inside a part, one piece of its drawing can be drawn aside on another drawing thread
//! (`draw_aside`, the view's own car: `race::car_part`), on a scratch of its own behind a
//! mutex, from the part's render states as the piece was handed aside; its drawing is put in
//! at its place in the part's order, as `draw_parts` puts a part's after the earlier ones.
//!
//! A list inside a part whose entries do not depend on each other (the course's sorted faces)
//! can be drawn in runs over the drawing threads the parts and the piece aside leave idle
//! (`list_runs`), each run on a scratch and a copy of the view of its own, put together in
//! the list's order.

use crate::errors::{BadInput, Refused};
use crate::memory::draw_order::{
    ASIDE_INDEX, COUNT, DrawScope, begin_list, ended, set_aside_turn, set_index,
};
use crate::memory::part_cell::PartTurn;
use crate::race::view_scratch::ViewScratch;
use crate::scene::scratch_store::{DrawingScratch, PartScratch};
use std::cell::Cell;
use std::sync::atomic::AtomicUsize;
use std::sync::{Mutex, OnceLock};

mod list_runs;
pub use list_runs::{draw_list_in_runs, list_runs_wanted};

// ---- the stores the parts read ----

/// Whether a race is on (`race::numbers_store::Numbers::in_race`), as part `part`'s drawing
/// reads it: its own copy (`set_game_part_racing` for the game's part, `draw_parts` for every
/// part when a scope opens).
pub fn in_race(part: &PartScratch) -> bool {
    part.in_race.is_set()
}

/// The game's part is told a race has begun or ended, on the game's thread, where the race flag
/// changes; every other part gets it when a drawing scope opens.
pub fn set_game_part_racing(scratch: &DrawingScratch, racing: bool) {
    scratch.game_scratch().in_race.set(racing);
}

/// The game's part is told the screen's figures, on the game's thread, where a screen mode is
/// set; every other part gets them when a drawing scope opens.
pub fn set_game_part_screen(scratch: &DrawingScratch, width: u32, height: u32, bits: u32) {
    for (cell, figure) in scratch
        .game_scratch()
        .screen_size
        .iter()
        .zip([width, height, bits])
    {
        cell.set(figure);
    }
}

/// The owners whose draw-shared stores a drawing reads: lent to the parts of a drawing scope
/// (`draw_parts`), or read from on the game's thread (`DrawReads::on_game_thread`).
#[derive(Clone, Copy)]
pub struct LentOwners<'a> {
    pub race: &'a crate::race::owner::Race,
    pub scene: &'a crate::scene::owner::Scene,
    pub system: &'a crate::system::owner::System,
    pub net: &'a crate::net::owner::Net,
}

impl<'a> LentOwners<'a> {
    /// The race's and the scene's owners of `o`.
    pub fn of(o: crate::owners::Owners<'a>) -> Self {
        Self {
            race: o.race,
            scene: o.scene,
            system: o.system,
            net: o.net,
        }
    }
}

/// The stores lent to the parts of one drawing scope: what every part reads and none writes.
/// Shared by reference with every part's run; a part reads them through `reads`.
pub struct Lent<'s> {
    /// The scratch the drawing works on (`Scene::scratch`).
    scratch: &'s DrawingScratch,
    light_tables:
        crate::memory::session::PhaseRead<'s, crate::scene::light_tables_store::LightTables>,
    // The scene's other drawing stores.
    light_work: crate::memory::session::PhaseRead<'s, crate::scene::light_work_store::LightWork>,
    draw_switches:
        crate::memory::session::PhaseRead<'s, crate::scene::draw_switches_store::DrawSwitches>,
    object_tables:
        crate::memory::session::PhaseRead<'s, crate::scene::object_tables_store::ObjectTables>,
    object_slots:
        crate::memory::session::PhaseRead<'s, crate::scene::object_slots_store::ObjectTables>,
    kind_textures:
        crate::memory::session::PhaseRead<'s, crate::scene::kind_texture_store::KindTextureTables>,
    texture_places:
        crate::memory::session::PhaseRead<'s, crate::scene::texture_places_store::TexturePlaces>,
    light_state: crate::memory::session::PhaseRead<'s, crate::scene::light_store::LightState>,
    // The scene's draw variables.
    scene_draw: crate::memory::session::PhaseRead<'s, crate::scene::draw_store::SceneDraw>,
    // The race's stores.
    level_grid: crate::memory::session::PhaseRead<'s, crate::race::level_grid_store::LevelGrid>,
    ground_probe:
        crate::memory::session::PhaseRead<'s, crate::physics::ground_probe_store::GroundProbe>,
    shadow_skids:
        crate::memory::session::PhaseRead<'s, crate::race::shadow_skids_store::ShadowSkids>,
    lights_and_gear:
        crate::memory::session::PhaseRead<'s, crate::race::groups_store::LightsAndGear>,
    finish: crate::memory::session::PhaseRead<'s, crate::drive::finish_store::Finish>,
    // The frame's work and the panel and overlay.
    frame_work: crate::memory::session::PhaseRead<'s, crate::race::frame_store::FrameWork>,
    frame_cars: &'s crate::race::frame_view::FrameCars,
    panel_and_overlay:
        crate::memory::session::PhaseRead<'s, crate::race::groups_store::PanelAndOverlay>,
    // The cars' blocks and the view variables.
    car_blocks: crate::memory::session::PhaseRead<'s, crate::race::blocks_store::CarBlocks>,
    view_and_records:
        crate::memory::session::PhaseRead<'s, crate::race::groups_store::ViewAndRecords>,
    // The system's screen.
    screen: crate::memory::session::PhaseRead<'s, crate::memory::session::Screen>,
    net_race: &'s rc2k_core::net::net_race::NetRaceSlot,
    // The course's groups.
    course_lists:
        crate::memory::session::PhaseRead<'s, crate::race::course_lists_store::CourseLists>,
    stage_prefs: crate::memory::session::PhaseRead<'s, crate::race::prefs_store::StagePrefs>,
    course_work: crate::memory::session::PhaseRead<'s, crate::race::course_store::CourseWork>,
    stage: &'s crate::race::stage_data_store::StageData,
    kind_records:
        crate::memory::session::PhaseRead<'s, crate::race::kind_records_store::KindRecords>,
    numbers: crate::memory::session::PhaseRead<'s, crate::race::numbers_store::Numbers>,
    switches: crate::memory::session::PhaseRead<'s, crate::race::switches_store::FrameSwitches>,
    vars: crate::memory::session::PhaseRead<'s, crate::race::vars_store::RaceVars>,
    view: crate::memory::session::PhaseRead<'s, crate::race::view_store::RaceView>,
    work: crate::memory::session::PhaseRead<'s, crate::race::work_store::RaceWork>,
    physics_tables:
        crate::memory::session::PhaseRead<'s, crate::physics::tables_store::PhysicsTables>,
    // The frame's matrices and the physics' contact work.
    matrices: crate::memory::session::PhaseRead<'s, crate::race::matrices_store::SceneMatrices>,
    contact: crate::memory::session::PhaseRead<'s, crate::physics::contact_store::ContactWork>,
    // The race's buffers the parts read.
    buffers: crate::memory::session::PhaseRead<'s, crate::race::buffers_store::BufferReads>,
}

impl<'s> Lent<'s> {
    fn new(scope: &'s DrawScope<'_>, owners: LentOwners<'s>) -> Self {
        use crate::memory::session::PhaseRead;
        let LentOwners {
            race,
            scene,
            system,
            net,
        } = owners;
        Self {
            scratch: &scene.scratch,
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
            frame_cars: &race.frame.cars,
            panel_and_overlay: PhaseRead::lend(&race.frame.panel_and_overlay, scope),
            // The cars' blocks and the view variables.
            car_blocks: PhaseRead::lend(&race.shared.car_blocks, scope),
            view_and_records: PhaseRead::lend(&race.frame.view_and_records, scope),
            // The system's screen.
            screen: PhaseRead::lend(&system.session.screen, scope),
            net_race: &net.race_slot,
            // The course's groups.
            course_lists: PhaseRead::lend(&race.course.lists, scope),
            stage_prefs: PhaseRead::lend(&race.course.stage_prefs, scope),
            course_work: PhaseRead::lend(&race.course.course, scope),
            stage: &race.course.stage,
            kind_records: PhaseRead::lend(race.course.kind_records_store(), scope),
            numbers: PhaseRead::lend(&race.course.numbers, scope),
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
            frame_cars: self.frame_cars,
            panel_and_overlay: &self.panel_and_overlay,
            // The cars' blocks and the view variables.
            car_blocks: &self.car_blocks,
            view_and_records: &self.view_and_records,
            // The system's screen.
            screen: &self.screen,
            net_race: self.net_race,
            // The course's groups.
            course_lists: &self.course_lists,
            stage_prefs: &self.stage_prefs,
            course_work: &self.course_work,
            stage: self.stage,
            kind_records: &self.kind_records,
            numbers: &self.numbers,
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

/// The draw-shared stores as a drawing reads them: a part's from the scope's `Lent` (handed to
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
    /// The cars' snapshot for this frame's drawing (`race::frame_view`).
    pub frame_cars: &'a crate::race::frame_view::FrameCars,
    /// The hud's panel and the race's overlay (`race::owner_frame::Frame::panel_and_overlay`).
    pub panel_and_overlay: &'a crate::race::groups_store::PanelAndOverlay,
    // The cars' blocks and the view variables.
    /// The cars' blocks (`race::owner_replay::Replay::car_blocks`).
    pub car_blocks: &'a crate::race::blocks_store::CarBlocks,
    /// The shared view variables and the view records
    /// (`race::owner_frame::Frame::view_and_records`).
    pub view_and_records: &'a crate::race::groups_store::ViewAndRecords,
    // The system's screen.
    /// The screen's figures (`system::owner::Session::screen`).
    pub screen: &'a crate::memory::session::Screen,
    /// The network game's slot: whether one is on, this machine's car and the players' count
    /// (`net::owner::Net::race_slot`), read live: the receiving thread can end the game.
    pub net_race: &'a rc2k_core::net::net_race::NetRaceSlot,
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
    /// The loaded stage's tables (`race::owner_course::Course::stage`).
    pub stage: &'a crate::race::stage_data_store::StageData,
    /// The course kinds' first and last records (`race::owner_course::Course::kind_records`).
    pub kind_records: &'a crate::race::kind_records_store::KindRecords,
    /// The game's numbers: its float constants, the replay's mode, `in_race`, the grip change
    /// (`race::owner_course::Course::numbers`).
    pub numbers: &'a crate::race::numbers_store::Numbers,
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
    /// The course's drawing stores as one value (`race::owner_course::CourseReads`).
    pub fn course(&self) -> crate::race::owner_course::CourseReads<'a> {
        crate::race::owner_course::CourseReads {
            course: self.course_work,
            lists: self.course_lists,
            stage_prefs: self.stage_prefs,
            numbers: self.numbers,
            kind_records: self.kind_records,
            stage: self.stage,
        }
    }

    /// The stores from their owners: on the game's thread, outside a drawing scope (the
    /// views drawn one after another, the overlays, the menus' objects).
    pub fn on_game_thread(owners: LentOwners<'a>) -> Self {
        let LentOwners {
            race,
            scene,
            system,
            net,
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
            frame_cars: &race.frame.cars,
            panel_and_overlay: race.frame.panel_and_overlay.get(),
            // The cars' blocks and the view variables.
            car_blocks: race.shared.car_blocks.get(),
            view_and_records: race.frame.view_and_records.get(),
            // The system's screen.
            screen: system.session.screen.get(),
            net_race: &net.race_slot,
            // The course's groups.
            course_lists: race.course.lists.get(),
            stage_prefs: race.course.stage_prefs.get(),
            course_work: race.course.course.get(),
            stage: &race.course.stage,
            kind_records: race.course.kind_records(),
            numbers: race.course.numbers.get(),
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

/// The most parts of a frame that can be drawn at once on this machine: its cores, up to the
/// drawing copies. The pool is sized for it, whatever the page asks for.
pub fn most_threads() -> usize {
    std::thread::available_parallelism()
        .map_or(1, usize::from)
        .min(COUNT)
}

/// How many parts of a frame may be drawn at once, as chosen (0: not chosen, the most). Read
/// at each frame, so a change takes effect at the next one.
static THREADS: AtomicUsize = AtomicUsize::new(0);

/// The choice in the options file: none for "as many as there are".
pub fn chosen_threads() -> Option<u32> {
    match THREADS.load(std::sync::atomic::Ordering::Relaxed) {
        0 => None,
        n => Some(n as u32),
    }
}

/// Chooses how many parts may be drawn at once (`None`: the most). A number past the most the
/// machine has is the most.
pub fn set_threads(chosen: Option<u32>) {
    THREADS.store(
        chosen.map_or(0, |n| (n.max(1) as usize).min(most_threads())),
        std::sync::atomic::Ordering::Relaxed,
    );
}

/// How many parts of a frame may be drawn at once now: 1 is one after another.
/// `RC2K_MIRROR_BESIDE=0` makes it 1 when nothing was chosen.
pub fn threads() -> usize {
    static ENV_OFF: OnceLock<bool> = OnceLock::new();
    match THREADS.load(std::sync::atomic::Ordering::Relaxed) {
        0 if *ENV_OFF
            .get_or_init(|| std::env::var_os("RC2K_MIRROR_BESIDE").is_some_and(|v| v == "0")) =>
        {
            1
        }
        0 => most_threads(),
        n => n.min(most_threads()),
    }
}

/// The rear-view mirror drawn beside its view (`draw_beside`, `race::draw_race_scene`): when two
/// parts may be drawn at once (`threads`), drawing and sounding the same as one after the
/// other. `RC2K_MIRROR_BESIDE=0` turns it off, any other value on.
pub fn mirror_beside() -> bool {
    threads() >= 2
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

/// The other drawing threads: one fewer than the most parts that can be drawn at once
/// (`most_threads`; the pages' choice limits the parts drawn, not the pool), up to the copies (the mirror beside its view uses one: the handheld at 25 to 30 frames a second; the
/// players' views and mirrors beside each other up to three).
fn pool() -> Option<&'static rayon::ThreadPool> {
    static POOL: OnceLock<Option<rayon::ThreadPool>> = OnceLock::new();
    POOL.get_or_init(|| {
        let all = most_threads();
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

/// Parts 0 .. `slots.len()` (`part(run, reads)`, at most `COUNT`, each on its own scratch and the
/// view slot `slots[n]`, which are distinct, reading the draw-shared stores of `owners` through
/// `reads`) drawn beside each other as `draw_beside` draws two: the first on the game's thread,
/// each other on a drawing thread, each from the working state as it is now; their drawing put
/// together and their changes put back in their order. `after(n)` runs on the game's thread once
/// part `n` is put back (part 0: once every part has ended), before the next is: what the drawing
/// does there goes after part `n`'s, on the working state as the parts so far left it, and no part
/// runs meanwhile (what it writes that has no copy per part, the cars' crumpled points, no part
/// reads while it changes).
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
        None => true,
    }) && threads() >= 4.min(most_threads()).max(2)
}

/// Parts 0 .. `slots.len()` of a drawing (`part`), part 0 on the game's thread and each other on a
/// drawing thread with its own copy of the working state, all set from the game's own at the
/// start; the other parts' noted drawing put after part 0's in their order, and each copy's
/// changes put into the game's own in that order, `after(n)` run after part `n`'s (part 0's
/// once all have ended). While the parts are drawn (a `DrawScope`) the draw-shared stores are lent
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
    let scratch_owner = &owners.scene.scratch;
    let first_cache = &scratch_owner.game_scratch().cache;
    let cache_start = crate::d3d::render_cache::current(first_cache);
    // The held matrix columns likewise (`scene::view_matrices::Held`), in the race's
    // matrices; every part, the first too, holds its own while the scope is open.
    let point_count = owners.race.course.course.get().scene_point_count.get();
    let matrices = &owners.race.frame.matrices;
    let columns_start = crate::scene::view_matrices::columns(matrices.get());
    begin_list();
    let control = crate::math::x87::control_word();
    // Whether a race is on, as the parts' noted drawing reads it inside the scope (`in_race`).
    let racing = owners.race.course.numbers.get().in_race.is_set();
    let screen = owners.system.session.screen.get();
    let screen_size = [screen.width.get(), screen.height.get(), screen.bits.get()];
    for scratch in scratch_owner.all_parts_on_game_thread() {
        scratch.in_race.set(racing);
        for (cell, figure) in scratch.screen_size.iter().zip(screen_size) {
            cell.set(figure);
        }
    }
    // The parts are drawn from here until every one has ended: the draw-shared stores
    // lent.
    let scope = DrawScope::open(scratch_owner);
    let lent = Lent::new(&scope, owners);
    let lent = &lent;
    let scope_ref = &scope;
    // Run `n` of the others, on the thread it is called on, with its own noted drawing.
    let other = |n: usize| {
        set_index(n);
        let turn = PartTurn::new(scope_ref, n);
        let scratch = scratch_owner.claim_part(n, &turn);
        let view = scratch_owner.claim_view(slots[n], &turn);
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
        let noted = crate::d3d::device::take_noted(scratch);
        set_index(0);
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
        let scratch = scratch_owner.claim_part(0, &turn);
        let view = scratch_owner.claim_view(slots[0], &turn);
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
        crate::d3d::device::put_noted(scratch_owner.game_scratch(), run.noted);
        crate::d3d::run_states::carry(first_cache, &run.states, &cache_start);
        crate::scene::view_matrices::merge_columns(matrices.get(), &columns_start, &run.columns);
        if result.is_ok() {
            // What it draws takes the merged states.
            crate::d3d::device::forget_states(scratch_owner.game_scratch());
            result = after(n + 1);
        }
    }
    // The game's thread drew run 0 with its own states: the next fan takes the merged ones.
    crate::d3d::device::forget_states(scratch_owner.game_scratch());
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
    car_part() && ((threads() >= 2 && pool().is_some()) || serial())
}

/// What a part may hand a piece of its own drawing to (`draw_aside`): the drawing threads (none
/// with `RC2K_DRAW_SERIAL`: the piece is then drawn on the part's own thread when it is joined,
/// on its copies all the same) and the stores lent to the scope.
#[derive(Clone, Copy)]
pub struct Helper<'a> {
    pool: Option<&'a rayon::ThreadPool>,
    lent: &'a Lent<'a>,
    /// How many drawing threads the part may use at once besides its own: the pool's without
    /// the other parts' (with `RC2K_DRAW_SERIAL`, as many as a list has runs' scratches).
    spare: usize,
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
            Some(p) if p.current_num_threads() >= parts && threads() > parts => Some(Self {
                pool: Some(p),
                lent,
                spare: p.current_num_threads().min(threads() - 1) + 1 - parts,
            }),
            None if serial() => Some(Self {
                pool: None,
                lent,
                spare: crate::memory::draw_order::LIST_RUNS,
            }),
            _ => None,
        }
    }
}

/// What a part's state was as it handed a piece aside: what the piece starts from.
struct AsideStart {
    point_count: u32,
    cache: crate::d3d::render_cache::Cache,
    columns: Option<crate::scene::view_matrices::Columns>,
    control: u16,
    in_race: bool,
    records_taken: bool,
    screen_size: [u32; 3],
}

/// What a piece drawn aside left: its result, its noted drawing (its fans before its own
/// setting of a state given the part's value of it at the piece's place, `Aside::join`) and
/// where it set the render states and the texture.
pub struct AsideRun {
    pub result: Result<(), Refused>,
    pub noted: crate::render::CommandList,
    states: crate::d3d::run_states::RunStates,
    /// The held matrix columns it left, if it held any.
    columns: Option<crate::scene::view_matrices::Columns>,
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
    scratch.records_taken.set(start.records_taken);
    for (cell, figure) in scratch.screen_size.iter().zip(start.screen_size) {
        cell.set(figure);
    }
    crate::math::x87::set_control_word(start.control);
    crate::d3d::run_states::begin_run(scratch, &start.cache);
    crate::scene::view_matrices::set_part_columns(scratch, start.columns);
    let result = piece(scratch, reads);
    let states = crate::d3d::run_states::end_run(scratch);
    let columns = crate::scene::view_matrices::end_part(scratch);
    scratch.points.end_run();
    let noted = crate::d3d::device::take_noted(scratch);
    AsideRun {
        result,
        noted,
        states,
        columns,
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
    /// The scratch the piece draws on lives in.
    scratch: &'a DrawingScratch,
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
                let scratch = self.scratch.aside();
                run_aside(&scratch, &self.start, lent.reads(), piece)
            }
        };
        let carried = crate::d3d::run_states::carried(&self.part.cache);
        crate::d3d::run_states::patch(&mut run.noted.commands, &run.states, &carried);
        let now = crate::d3d::render_cache::current(&self.part.cache);
        if crate::d3d::run_states::drew_before_texture(&run.noted.commands, &run.states)
            && !crate::d3d::render_cache::same_texture(&now, &self.start.cache)
        {
            crate::program_log::log(
                "workers: a piece drawn aside drew before setting its texture (not as one thread)",
            );
        }
        Ok(run)
    }

    /// `run`'s drawing put after the part's so far, and the render states and the texture it
    /// set into the part's cache.
    pub fn put(&self, run: AsideRun) -> Result<(), Refused> {
        debug_assert!(
            std::ptr::eq(self.part, self.scratch.game_scratch()),
            "a piece drawn aside of a part other than the first"
        );
        crate::d3d::device::put_noted(self.part, run.noted);
        crate::d3d::run_states::carry_set(&self.part.cache, &run.states);
        // The point records it wrote, as the part would have them after it; the piece's own
        // set back to 0 for the next piece.
        {
            let scratch = self.scratch.aside();
            crate::scene::point_records::put_written(&scratch, self.part);
            crate::scene::point_records::clear_written(&scratch);
        }
        crate::d3d::device::forget_states(self.part);
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
            crate::scene::point_records::clear_written(&self.scratch.aside());
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
        records_taken: part.records_taken.is_set(),
        screen_size: part
            .screen_size
            .each_ref()
            .map(crate::memory::session::PlainDword::get),
    };
    set_aside_turn(false);
    let Some(pool) = helper.pool else {
        let aside = Aside {
            part,
            scratch: helper.lent.scratch,
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
        scratch: helper.lent.scratch,
        start,
        ran: Some(ran),
        later: std::cell::RefCell::new(None),
        joined: Cell::new(false),
    };
    let lent = helper.lent;
    let begin = &aside.start;
    pool.in_place_scope(|s| {
        s.spawn(move |_| {
            set_index(ASIDE_INDEX);
            let run = {
                let scratch = lent.scratch.aside();
                run_aside(&scratch, begin, lent.reads(), piece)
            };
            set_index(0);
            let _ = send.send(run);
        });
        let _turn = GiveTurn;
        let r = rest(&aside);
        aside.finish();
        r
    })
}
