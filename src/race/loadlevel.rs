//! `race::loadlevel_init` (0x44426F: the stage's loading proper, step by step,
//! each step's word logged ("LOADLEVEL_INIT : ...") with the log indented round it and the
//! loading bar moved on (the step's mark, then the callback `race::loading_bar_frame`, called with the mark).
//! The pace notes are set up by `race::init_pace_notes`. A step's refusal is the function's.

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::scene::file_numbers_store::texts;
const BAR: u32 = 0x4467C7; // `race::loading_bar_frame`: the loading bar's 3D frame
const BAR_DONE: u32 = 0x4467C6; // nullsub_8

/// The step words ("LOADLEVEL_INIT : start" .. "finis"), in order.
const WORDS: [&std::ffi::CStr; 28] = [
    c"LOADLEVEL_INIT : start",
    c"LOADLEVEL_INIT : objects_init",
    c"LOADLEVEL_INIT : alloc coursemem",
    c"LOADLEVEL_INIT : load texture info",
    c"LOADLEVEL_INIT : load prefs",
    c"LOADLEVEL_INIT : load errorlist",
    c"LOADLEVEL_INIT : init ed textures",
    c"LOADLEVEL_INIT : texturesalloc",
    c"LOADLEVEL_INIT : initgametexinfbuf",
    c"LOADLEVEL_INIT : locacurrcourse",
    c"LOADLEVEL_INIT : clearsysplates",
    c"LOADLEVEL_INIT : reloadobjects",
    c"LOADLEVEL_INIT : setplatesall",
    c"LOADLEVEL_INIT : loadgamemap",
    c"LOADLEVEL_INIT : loadgamefonts",
    c"LOADLEVEL_INIT : init mark",
    c"LOADLEVEL_INIT : refresh init",
    c"LOADLEVEL_INIT : skidmarks init",
    c"LOADLEVEL_INIT : render3d init",
    c"LOADLEVEL_INIT : render3d setcolours",
    c"LOADLEVEL_INIT : render3d loadlightsdata",
    c"LOADLEVEL_INIT : render3d initmud",
    c"LOADLEVEL_INIT : getmatcrash",
    c"SHAUNINITEVERYTHING : store norms",
    c"LOADLEVEL_INIT : initcheckpoints",
    c"loadlevel : init pacenotes",
    c"LOADLEVEL_INIT : initoverlay",
    c"LOADLEVEL_INIT : finis",
];

/// Where the variables kept apart are: the log's indent and switch,
/// `g_inRace`, and the loading bar's callback.
#[derive(Clone, Copy)]
pub struct LoadVars {
    pub bar: u32,
}

impl LoadVars {
    pub fn game() -> Self {
        Self { bar: BAR }
    }
}

/// The original's `loadlevel_init`, step by step.
struct Steps<'a> {
    race: &'a Race,
    o: crate::owners::Owners<'static>,
    v: LoadVars,
    word: usize,
}

impl Steps<'_> {
    /// Logs the next word.
    fn log(&mut self) {
        crate::logfile::log_write::log_nul(&WORDS[self.word], true, 1);
        self.word += 1;
    }

    /// `pushf`; `g_logIndent` +- 3; `popf`.
    fn indent(&mut self, by: i32) {
        crate::logfile::write_line::state::INDENT
            .atomic()
            .fetch_add(by as u32, std::sync::atomic::Ordering::Relaxed);
    }

    /// `pusha`; `bar_share` the mark; `call [`bar_function`]`; `popa`. The callback is the
    /// loading bar `loadlevel_init` puts there before its first mark (it is put away after
    /// the last).
    fn progress(&mut self, mark: u32) -> Result<(), Refused> {
        let race = self.race;
        let shared = &race.shared;
        shared.vars.bar_share.set(mark);
        if shared.vars.bar_function.get() != self.v.bar {
            return Err(BadInput("the loading bar's callback is not the loading bar").into());
        }
        crate::race::loading_bar_frame::loading_bar_frame(
            race,
            self.o.system,
            self.o.scene,
            self.o.gfx,
            self.o.d3d,
        )
    }

    /// A step's usual frame: the word, the indent in, the call, the mark, the indent out.
    fn step(
        &mut self,
        call: &mut dyn FnMut() -> Result<(), Refused>,
        mark: Option<u32>,
    ) -> Result<(), Refused> {
        self.log();
        self.indent(3);
        call()?;
        if let Some(m) = mark {
            self.progress(m)?;
        }
        self.indent(-3);
        Ok(())
    }
}

/// `race::loadlevel_init` (0x44426F): "start"; `g_usePackedMaps` 1, `MapVars.packed_unless_editor` 1 but with
/// `editor_mode`; `g_stageDirection` `direction` (flipped for stages 0x1F, 0x22, 0x51);
/// `g_inRace` 1; the loading bar's callback `race::loading_bar_frame` (mark 0); then the steps
/// (`CALLEES` in order, each with its word and mark: objects, course memory (2, 4), texture info
/// and the course number (6), prefs (7), error list (8), ed textures (0xA), textures (0xE), texture
/// infos (0xF), the course (0x13), plates cleared (0x14), objects (0x1E; `bar_span`/`38`
/// 0x1E/0x50), plates (0x50), the map (0x5A), fonts (0x5C), init mark with the log off (0x5E),
/// refresh, skid marks (0x5F), render3d (0x60), colours (0x62), lights (0x63), mud (0x64 twice);
/// the callback `nullsub_8`; the crash matrix, the norms; without `Finish.no_place` checkpoints, pace
/// notes, overlay; `marker_object` the object of kind 0x7A (else 0x79); `camera_last_frame`/`D4` 0;
/// "finis".
pub fn loadlevel_init(
    race: &'static Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    let (scene, system, gfx) = (o.scene, o.system, o.gfx);
    use crate::race::level_init as level;
    use crate::race::stage_prefs;
    let v = LoadVars::game();
    let mut s = Steps {
        race,
        o,
        v,
        word: 0,
    };
    s.log();
    s.indent(3);
    race.frame.map().use_packed.set(1u32);
    race.frame
        .map()
        .packed_unless_editor
        .set((race.shared.switches.editor_mode.get() == 0) as u32);
    let mut direction: u32 = race.shared.vars.direction.get();
    if [0x1F, 0x22, 0x51].contains(&race.shared.vars.stage_number.get()) {
        direction ^= 1;
    }
    race.shared.work.stage_direction.set(direction);
    race.course.numbers.get().in_race.set(true);
    race.shared.vars.bar_function.set(v.bar);
    s.progress(0)?;
    s.indent(-3);
    s.step(
        &mut || {
            crate::scene::set_up_object_memory::set_up_object_memory(race, scene)?;
            Ok(())
        },
        None,
    )?;
    // alloc coursemem: marks 2 and 4 (the `cmp `editor_mode`, 0 / jz $+2` between
    // them changes nothing).
    s.log();
    s.indent(3);
    // CourseLists.course_memory_list[0]: the course's MemoryList.
    level::allocate_course_memory(&race.course, &race.course.lists.get().course_memory_list)?;
    s.progress(2)?;
    s.progress(4)?;
    s.indent(-3);
    // load texture info and the course number.
    s.log();
    s.indent(3);
    level::load_texture_info(&race.course, scene)?;
    level::set_course_number(race.shared.vars.stage_number.get(), race, scene)?;
    s.progress(6)?;
    s.indent(-3);
    s.step(&mut || stage_prefs::load_prefs(race), Some(7))?;
    s.step(&mut || level::load_error_list(&race.frame, scene), Some(8))?;
    s.step(
        &mut || crate::scene::texture_levels::init_texture_levels(race, o),
        Some(0xA),
    )?;
    s.step(
        &mut || crate::scene::set_up_textures::set_up_textures(&race.shared, scene, o.gfx, o.d3d),
        Some(0xE),
    )?;
    s.step(&mut || level::build_texture_infos(race), Some(0xF))?;
    s.step(
        &mut || crate::race::course_data::locate_current_course(race, scene),
        Some(0x13),
    )?;
    s.step(
        &mut || {
            crate::scene::reset_texture_places::reset_texture_places(scene);
            Ok(())
        },
        Some(0x14),
    )?;
    // reloadobjects.
    s.log();
    s.indent(3);
    crate::race::objects::reload_objects(race, scene)?;
    s.progress(0x1E)?;
    race.shared.vars.bar_span[0].set(0x1E); // RaceVars.bar_span[0]
    race.shared.vars.bar_span[1].set(0x50); // RaceVars.bar_span[1]
    s.indent(-3);
    s.step(
        &mut || crate::race::set_all_plates::set_all_plates(race, system, gfx, o.d3d, o.net, scene),
        Some(0x50),
    )?;
    // loadgamemap.
    s.log();
    s.indent(3);
    crate::race::game_map::load_game_map(
        &race.frame,
        &scene.file_numbers().text_at(texts::MAP_PATH),
        &scene.file_numbers().text_at(texts::MAP_KEY),
        system,
        gfx,
    )?;
    s.progress(0x5A)?;
    s.indent(-3);
    s.step(
        &mut || crate::gfx::build_game_fonts::build_game_fonts_in_game(gfx, system, o.strings),
        Some(0x5C),
    )?;
    // init mark, with the log off.
    s.log();
    s.indent(3);
    let enabled = crate::logfile::write_line::state::ENABLED
        .atomic()
        .swap(0, std::sync::atomic::Ordering::Relaxed);
    crate::effects::init_mark_step::init_mark_step(race, o.effects, scene, gfx, system, o.net)?;
    crate::logfile::write_line::state::ENABLED.set(enabled);
    s.progress(0x5E)?;
    s.indent(-3);
    // refresh init and the refresh flags.
    s.log();
    s.indent(3);
    level::reset_refresh();
    let flags: u32 = race.course.stage_prefs.get().refresh_flags_a.get();
    scene.lights.work.get().marks_mask.set((flags & 2) >> 1);
    for car in &race.car_blocks().cars[..8] {
        car.lights_on.set(flags & 1);
    }
    scene
        .lights
        .work
        .get()
        .lamps_mask
        .set(race.course.stage_prefs.get().refresh_flags_b.get() & 1);
    s.indent(-3);
    s.step(
        &mut || {
            stage_prefs::init_skidmarks(race)?;
            Ok(())
        },
        Some(0x5F),
    )?;
    // render3d init: `scene::allocate_point_records`.
    s.step(
        &mut || {
            crate::scene::allocate_point_records::allocate_point_records_in_game(&race.course)?;
            Ok(())
        },
        Some(0x60),
    )?;
    s.step(
        &mut || {
            crate::scene::fill_scene_light_tables::fill_scene_light_tables(race, scene)?;
            Ok(())
        },
        Some(0x62),
    )?;
    s.step(&mut || stage_prefs::load_lights_data(race), Some(0x63))?;
    // initmud: mark 0x64 twice, then the callback put away.
    s.log();
    s.indent(3);
    crate::race::mud::init_mud(race, scene, gfx, o.net, system)?;
    s.progress(0x64)?;
    s.progress(0x64)?;
    race.shared.vars.bar_function.set(BAR_DONE);
    s.indent(-3);
    s.step(
        &mut || {
            stage_prefs::get_mat_crash(
                race.engine_tables(),
                &race.frame.matrices,
                race.shared.car(),
                scene,
            );
            Ok(())
        },
        None,
    )?;
    // store norms; without `Finish.no_place` the checkpoints, the pace notes, the overlay.
    s.log();
    s.indent(3);
    crate::race::norms::store_norms(race, &mut |a, b, c| {
        crate::race::norms::face_normal(race, scene, a, b, c)
    })?;
    if race.replay.finish.get().no_place.get() == 0 {
        s.indent(-3);
        s.step(&mut || level::init_checkpoints(race), None)?;
        s.step(
            &mut || crate::race::init_pace_notes::init_pace_notes(race),
            None,
        )?;
        s.log();
        s.indent(3);
        crate::race::init_overlay::init_overlay(race, scene)?;
    } else {
        s.word += 3;
    }
    use crate::scene::find_object_of_kind::find_object_of_kind;
    let mut marker = find_object_of_kind(0x7A, scene);
    race.shared.switches.marker_object.set(marker);
    if marker == 0 {
        marker = find_object_of_kind(0x79, scene);
        race.shared.switches.marker_object.set(marker);
    }
    // `camera_last_frame` / D4.
    for cell in &race.replay_work().camera_last_frame {
        cell.set(0);
    }
    s.indent(-3);
    s.log();
    Ok(())
}
