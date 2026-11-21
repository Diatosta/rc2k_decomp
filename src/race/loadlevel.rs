//! `race::loadlevel_init`: the stage's loading proper, step by step. Each step's word is logged
//! ("LOADLEVEL_INIT : ...") with the log indented round it, and the loading bar is moved on by
//! the step's mark. The pace notes are set up by `race::init_pace_notes`. A step's refusal is the
//! function's.

use crate::errors::Refused;
use crate::race::loading_bar_frame::Progress;
use crate::race::owner::Race;
use crate::scene::file_numbers_store::texts;

/// The load's steps, in the order their words are logged.
#[derive(Clone, Copy)]
enum Word {
    Start,
    ObjectsInit,
    AllocCourseMemory,
    LoadTextureInfo,
    LoadPrefs,
    LoadErrorList,
    InitEdTextures,
    TexturesAlloc,
    InitGameTexInfBuf,
    LocateCurrentCourse,
    ClearSystemPlates,
    ReloadObjects,
    SetPlatesAll,
    LoadGameMap,
    LoadGameFonts,
    InitMark,
    RefreshInit,
    SkidmarksInit,
    Render3dInit,
    Render3dSetColours,
    Render3dLoadLightsData,
    Render3dInitMud,
    GetMatCrash,
    StoreNorms,
    InitCheckpoints,
    InitPaceNotes,
    InitOverlay,
    Finis,
}

impl Word {
    /// The step's line in the log.
    fn text(self) -> &'static std::ffi::CStr {
        match self {
            Word::Start => c"LOADLEVEL_INIT : start",
            Word::ObjectsInit => c"LOADLEVEL_INIT : objects_init",
            Word::AllocCourseMemory => c"LOADLEVEL_INIT : alloc coursemem",
            Word::LoadTextureInfo => c"LOADLEVEL_INIT : load texture info",
            Word::LoadPrefs => c"LOADLEVEL_INIT : load prefs",
            Word::LoadErrorList => c"LOADLEVEL_INIT : load errorlist",
            Word::InitEdTextures => c"LOADLEVEL_INIT : init ed textures",
            Word::TexturesAlloc => c"LOADLEVEL_INIT : texturesalloc",
            Word::InitGameTexInfBuf => c"LOADLEVEL_INIT : initgametexinfbuf",
            Word::LocateCurrentCourse => c"LOADLEVEL_INIT : locacurrcourse",
            Word::ClearSystemPlates => c"LOADLEVEL_INIT : clearsysplates",
            Word::ReloadObjects => c"LOADLEVEL_INIT : reloadobjects",
            Word::SetPlatesAll => c"LOADLEVEL_INIT : setplatesall",
            Word::LoadGameMap => c"LOADLEVEL_INIT : loadgamemap",
            Word::LoadGameFonts => c"LOADLEVEL_INIT : loadgamefonts",
            Word::InitMark => c"LOADLEVEL_INIT : init mark",
            Word::RefreshInit => c"LOADLEVEL_INIT : refresh init",
            Word::SkidmarksInit => c"LOADLEVEL_INIT : skidmarks init",
            Word::Render3dInit => c"LOADLEVEL_INIT : render3d init",
            Word::Render3dSetColours => c"LOADLEVEL_INIT : render3d setcolours",
            Word::Render3dLoadLightsData => c"LOADLEVEL_INIT : render3d loadlightsdata",
            Word::Render3dInitMud => c"LOADLEVEL_INIT : render3d initmud",
            Word::GetMatCrash => c"LOADLEVEL_INIT : getmatcrash",
            Word::StoreNorms => c"SHAUNINITEVERYTHING : store norms",
            Word::InitCheckpoints => c"LOADLEVEL_INIT : initcheckpoints",
            Word::InitPaceNotes => c"loadlevel : init pacenotes",
            Word::InitOverlay => c"LOADLEVEL_INIT : initoverlay",
            Word::Finis => c"LOADLEVEL_INIT : finis",
        }
    }
}

/// The stage's loading, step by step: the log and the loading bar.
struct Steps<'a> {
    progress: &'a mut dyn Progress,
}

impl Steps<'_> {
    /// Logs the step's word.
    fn log(&mut self, word: Word) {
        crate::logfile::load_times::phase(word.text().to_str().unwrap_or("?"));
        crate::logfile::log_write::log_nul(&word.text(), true, 1);
    }

    /// Moves the log's indent by `by`.
    fn indent(&mut self, by: i32) {
        crate::logfile::write_line::state::INDENT
            .atomic()
            .fetch_add(by as u32, std::sync::atomic::Ordering::Relaxed);
    }

    /// Moves the loading bar to `mark` percent and shows a frame.
    fn mark(&mut self, mark: u32) -> Result<(), Refused> {
        self.progress.mark(mark)
    }

    /// A step's usual frame: the word, the indent in, the call, the mark, the indent out.
    fn step(
        &mut self,
        word: Word,
        call: &mut dyn FnMut() -> Result<(), Refused>,
        mark: Option<u32>,
    ) -> Result<(), Refused> {
        self.log(word);
        self.indent(3);
        call()?;
        if let Some(m) = mark {
            self.mark(m)?;
        }
        self.indent(-3);
        Ok(())
    }
}

/// Loads the stage: logs "start"; sets `packed_unless_editor` 1
/// (but 0 with `editor_mode`), `stage_direction` from `direction` (flipped for stages 0x1F, 0x22,
/// 0x51) and `in_race` 1; marks the loading bar 0;
/// then runs the steps, each with its word and mark: objects, course memory (2, 4), texture info
/// and the course number (6), prefs (7), error list (8), ed textures (0xA), textures (0xE),
/// texture infos (0xF), the course (0x13), plates cleared (0x14), objects (0x1E; the texture marks then run from
/// 0x1E to 0x50), plates (0x50), the map (0x5A), fonts (0x5C), init mark with the log off (0x5E),
/// refresh, skid marks (0x5F), render3d (0x60), colours (0x62), lights (0x63), mud (0x64 twice);
/// the crash matrix, the norms; without `Finish.no_place` checkpoints,
/// pace notes, overlay; `marker_object` the object of kind 0x7A (else 0x79);
/// `camera_last_frame` 0; "finis".
pub fn loadlevel_init(
    race: &'static Race,
    o: crate::owners::Owners<'static>,
    progress: &mut dyn Progress,
) -> Result<(), Refused> {
    let mut s = Steps { progress };
    begin_level(&mut s, race, o)?;
    load_course_data(&mut s, race, o)?;
    load_objects_and_map(&mut s, race, o)?;
    init_marks_and_lights(&mut s, race, o)?;
    finish_level(&mut s, race, o)?;
    Ok(())
}

/// The level begun: the map's and the stage's variables, the loading bar's first mark.
fn begin_level(
    s: &mut Steps<'_>,
    race: &'static Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    let scene = o.scene;
    s.log(Word::Start);
    s.indent(3);
    race.frame.map().use_packed.set(1u32);
    race.frame
        .map()
        .packed_unless_editor
        .set((race.shared.switches.get().editor_mode.get() == 0) as u32);
    let mut direction: u32 = race.shared.vars.get().direction.get();
    if [0x1F, 0x22, 0x51].contains(&race.shared.vars.get().stage_number.get()) {
        direction ^= 1;
    }
    race.shared.work.get().stage_direction.set(direction);
    race.course.numbers.get().in_race.set(true);
    crate::scene::workers::set_game_part_racing(&scene.scratch, true);
    o.audio.racing.set(true);
    s.mark(0)?;
    s.indent(-3);
    Ok(())
}

/// The object memory, the course memory, the texture information, the prefs, the textures and the course.
fn load_course_data(
    s: &mut Steps<'_>,
    race: &'static Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    let scene = o.scene;
    use crate::race::level_init as level;
    use crate::race::stage_prefs;
    s.step(
        Word::ObjectsInit,
        &mut || {
            crate::scene::set_up_object_memory::set_up_object_memory(race, scene)?;
            Ok(())
        },
        None,
    )?;
    // The course memory: marks 2 and 4.
    s.log(Word::AllocCourseMemory);
    s.indent(3);
    // CourseLists.course_memory_list[0]: the course's MemoryList.
    level::allocate_course_memory(&race.course, &race.course.lists.get().course_memory_list)?;
    s.mark(2)?;
    s.mark(4)?;
    s.indent(-3);
    // load texture info and the course number.
    s.log(Word::LoadTextureInfo);
    s.indent(3);
    level::load_texture_info(&race.course, scene)?;
    level::set_course_number(race.shared.vars.get().stage_number.get(), race, scene)?;
    s.mark(6)?;
    s.indent(-3);
    s.step(
        Word::LoadPrefs,
        &mut || stage_prefs::load_prefs(race),
        Some(7),
    )?;
    s.step(
        Word::LoadErrorList,
        &mut || level::load_error_list(&race.frame, scene),
        Some(8),
    )?;
    s.step(
        Word::InitEdTextures,
        &mut || crate::scene::texture_levels::init_texture_levels(race, o),
        Some(0xA),
    )?;
    s.step(
        Word::TexturesAlloc,
        &mut || {
            crate::scene::set_up_textures::set_up_textures(&race.shared, scene, o.gfx, &o.gfx.d3d)
        },
        Some(0xE),
    )?;
    s.step(
        Word::InitGameTexInfBuf,
        &mut || level::build_texture_infos(race),
        Some(0xF),
    )?;
    s.step(
        Word::LocateCurrentCourse,
        &mut || crate::race::course_data::locate_current_course(race, scene),
        Some(0x13),
    )?;
    s.step(
        Word::ClearSystemPlates,
        &mut || {
            crate::scene::reset_texture_places::reset_texture_places(scene);
            Ok(())
        },
        Some(0x14),
    )?;
    Ok(())
}

/// The objects, their plates, the game map and the fonts.
fn load_objects_and_map(
    s: &mut Steps<'_>,
    race: &'static Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    let (scene, system, gfx) = (o.scene, o.system, o.gfx);
    // reloadobjects.
    s.log(Word::ReloadObjects);
    s.indent(3);
    crate::race::objects::reload_objects(race, scene)?;
    s.mark(0x1E)?;
    s.progress.set_span(0x1E, 0x50);
    s.indent(-3);
    s.log(Word::SetPlatesAll);
    s.indent(3);
    crate::race::set_all_plates::set_all_plates(o, &mut *s.progress)?;
    s.mark(0x50)?;
    s.indent(-3);
    // loadgamemap.
    s.log(Word::LoadGameMap);
    s.indent(3);
    let game_map = crate::race::game_map::load_game_map(
        &race.frame,
        &scene.path_text(texts::MAP_PATH),
        &scene.path_text(texts::MAP_KEY),
        system,
        gfx,
    )?;
    crate::race::game_map::store_game_map(race, game_map)?;
    s.mark(0x5A)?;
    s.indent(-3);
    s.step(
        Word::LoadGameFonts,
        &mut || crate::gfx::build_game_fonts::build_game_fonts(gfx, system, o.strings),
        Some(0x5C),
    )?;
    Ok(())
}

/// The marks, the refresh flags, the skid marks, the 3D's tables, the lights and the mud.
fn init_marks_and_lights(
    s: &mut Steps<'_>,
    race: &'static Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    let (scene, system, gfx) = (o.scene, o.system, o.gfx);
    use crate::race::level_init as level;
    use crate::race::stage_prefs;
    // init mark, with the log off.
    s.log(Word::InitMark);
    s.indent(3);
    let enabled = crate::logfile::write_line::state::ENABLED
        .atomic()
        .swap(0, std::sync::atomic::Ordering::Relaxed);
    crate::effects::init_mark_step::init_mark_step(
        race,
        o.race.effects.get(),
        scene,
        gfx,
        system,
        o.net,
    )?;
    crate::logfile::write_line::state::ENABLED.set(enabled);
    s.mark(0x5E)?;
    s.indent(-3);
    // refresh init and the refresh flags.
    s.log(Word::RefreshInit);
    s.indent(3);
    level::reset_refresh(&scene.scratch);
    let flags: u32 = race.course.stage_prefs.get().refresh_flags_a.get();
    scene.light_work_mut().marks_mask = (flags & 2) >> 1;
    for car in &race.car_blocks().cars[..8] {
        car.lights_on.set(flags & 1);
    }
    scene.light_work_mut().lamps_mask = race.course.stage_prefs.get().refresh_flags_b.get() & 1;
    s.indent(-3);
    s.step(
        Word::SkidmarksInit,
        &mut || {
            stage_prefs::init_skidmarks(race)?;
            Ok(())
        },
        Some(0x5F),
    )?;
    // render3d init: `scene::allocate_point_records`.
    s.step(
        Word::Render3dInit,
        &mut || {
            crate::scene::allocate_point_records::allocate_point_records_in_game(
                &scene.scratch,
                &race.course,
            )?;
            Ok(())
        },
        Some(0x60),
    )?;
    s.step(
        Word::Render3dSetColours,
        &mut || {
            crate::scene::fill_scene_light_tables::fill_scene_light_tables(race, scene)?;
            Ok(())
        },
        Some(0x62),
    )?;
    s.step(
        Word::Render3dLoadLightsData,
        &mut || stage_prefs::load_lights_data(race),
        Some(0x63),
    )?;
    // initmud: mark 0x64 twice, then the callback put away.
    s.log(Word::Render3dInitMud);
    s.indent(3);
    crate::race::mud::init_mud(race, scene, gfx, system, &o.net.race_slot)?;
    s.mark(0x64)?;
    s.mark(0x64)?;
    s.indent(-3);
    Ok(())
}

/// The crash matrix, the norms, the checkpoints, the pace notes and the overlay, the marker object, and the camera's last frames.
fn finish_level(
    s: &mut Steps<'_>,
    race: &'static Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    let scene = o.scene;
    use crate::race::level_init as level;
    use crate::race::stage_prefs;
    s.step(
        Word::GetMatCrash,
        &mut || {
            stage_prefs::get_mat_crash(
                race.engine_tables(),
                race.frame.matrices.get(),
                race.shared.car(),
                scene,
            );
            Ok(())
        },
        None,
    )?;
    // store norms; without `Finish.no_place` the checkpoints, the pace notes, the overlay.
    s.log(Word::StoreNorms);
    s.indent(3);
    crate::race::norms::store_norms(race, scene)?;
    if race.replay.no_place() == 0 {
        s.indent(-3);
        s.step(
            Word::InitCheckpoints,
            &mut || level::init_checkpoints(race),
            None,
        )?;
        s.step(
            Word::InitPaceNotes,
            &mut || crate::race::init_pace_notes::init_pace_notes(race),
            None,
        )?;
        s.log(Word::InitOverlay);
        s.indent(3);
        crate::race::init_overlay::init_overlay(race, scene)?;
    }
    use crate::scene::find_object_of_kind::find_object_of_kind;
    let mut marker = find_object_of_kind(0x7A, scene);
    race.shared.switches.get().marker_object.set(marker);
    if marker == 0 {
        marker = find_object_of_kind(0x79, scene);
        race.shared.switches.get().marker_object.set(marker);
    }
    // `camera_last_frame` cleared.
    {
        let mut work = race.replay_work_mut();
        work.camera_last_frame = [0; 2];
        work.camera_last_frame_extra = [0; 2];
    }
    s.indent(-3);
    s.log(Word::Finis);
    Ok(())
}
