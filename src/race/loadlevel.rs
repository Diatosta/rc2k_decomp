//! `race::loadlevel_init` (0x44426F, IDA 2026-10-17): the stage's loading proper, step by step,
//! each step's word logged ("LOADLEVEL_INIT : ...") with the log indented round it and the
//! loading bar moved on (`dword_623700` the step's mark, then the callback in `dword_623704`,
//! `sub_4467C7`, called with esi it). The pace notes are `race::init_pace_notes` since
//! 2026-10-19. A step's refusal is the function's (2026-10-23: no longer handed to the step's
//! address).

use crate::errors::{BadInput, Refused};
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::game_map::MAP;
use crate::race::numbers_store::NUMBERS;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::replay_work_store::REPLAY_WORK;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::vars_store::RACE_VARS;
use crate::race::work_store::RACE_WORK;
use crate::scene::file_numbers_store::{FILE_NUMBERS, texts};
use crate::scene::light_work_store::LIGHT_WORK;
const BAR: u32 = 0x4467C7; // sub_4467C7: the loading bar's 3D frame
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
struct Steps {
    v: LoadVars,
    word: usize,
}

impl Steps {
    /// `pusha`; eax the next word; the log; `popa`.
    fn log(&mut self) {
        crate::logfile::log_write::log_nul(&WORDS[self.word], true, 1);
        self.word += 1;
    }

    /// `pushf`; `g_logIndent` +- 3; `popf`.
    fn indent(&mut self, by: i32) {
        crate::logfile::write_line::state::INDENT
            .fetch_add(by, std::sync::atomic::Ordering::Relaxed);
    }

    /// `pusha`; `dword_623700` the mark; `call [dword_623704]`; `popa`. The callback is the
    /// loading bar `loadlevel_init` puts there before its first mark (it is put away after
    /// the last).
    fn progress(&mut self, mark: u32) -> Result<(), Refused> {
        RACE_VARS.bar_share.set_bits(mark);
        if RACE_VARS.bar_function.bits() != self.v.bar {
            return Err(BadInput("the loading bar's callback is not the loading bar").into());
        }
        crate::race::loading_bar_frame::loading_bar_frame()
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

/// `race::loadlevel_init` (0x44426F): "start"; `g_usePackedMaps` 1, `dword_5BE9B9` 1 but
/// with `dword_621514`; `g_stageDirection` `dword_623854` (flipped for stages 0x1F, 0x22,
/// 0x51); `g_inRace` 1; the loading bar's callback `sub_4467C7` (mark 0); then the steps
/// (`CALLEES` in order, each with its word and mark: objects, course memory (2, 4),
/// texture info and the course number (6), prefs (7), error list (8), ed textures (0xA),
/// textures (0xE), texture infos (0xF), the course (0x13), plates cleared (0x14), objects
/// (0x1E; `dword_623634`/`38` 0x1E/0x50), plates (0x50), the map (0x5A), fonts (0x5C),
/// init mark with the log off (0x5E), refresh (and the refresh flags from
/// `dword_609354`/`50` into `dword_62DAD0`, the blocks' +0x84, `dword_62DACC`), skid
/// marks (0x5F), render3d (0x60), colours (0x62), lights (0x63), mud (0x64 twice); the
/// callback `nullsub_8`; the crash matrix, the norms; without `dword_64BB18` checkpoints,
/// pace notes, overlay; `dword_621508` the object of kind 0x7A (else 0x79);
/// `dword_719FD0`/`D4` 0; "finis". What the original leaves in the registers is read by
/// no caller (`race::load_stage` calls on; conversion phase 3, 2026-10-24).
pub fn loadlevel_init() -> Result<(), Refused> {
    use crate::race::level_init as level;
    use crate::race::stage_prefs;
    let v = LoadVars::game();
    let mut s = Steps { v, word: 0 };
    s.log();
    s.indent(3);
    MAP.use_packed.set(1u32);
    MAP.packed_2
        .set((FRAME_SWITCHES.editor_mode.bits() == 0) as u32);
    let mut direction: u32 = RACE_VARS.direction.bits();
    if [0x1F, 0x22, 0x51].contains(&RACE_VARS.stage_number.bits()) {
        direction ^= 1;
    }
    RACE_WORK.stage_direction.set(direction);
    NUMBERS.in_race.set(1);
    RACE_VARS.bar_function.set_bits(v.bar);
    s.progress(0)?;
    s.indent(-3);
    s.step(
        &mut || {
            crate::scene::set_up_object_memory::set_up_object_memory()?;
            Ok(())
        },
        None,
    )?;
    // alloc coursemem: marks 2 and 4 (the `cmp dword_621514, 0 / jz $+2` between
    // them changes nothing).
    s.log();
    s.indent(3);
    // stru_5C6D50: the course's MemoryList.
    level::allocate_course_memory(
        &crate::race::course_lists_store::COURSE_LISTS.course_memory_list,
    )?;
    s.progress(2)?;
    s.progress(4)?;
    s.indent(-3);
    // load texture info and the course number.
    s.log();
    s.indent(3);
    level::load_texture_info()?;
    level::set_course_number(RACE_VARS.stage_number.bits())?;
    s.progress(6)?;
    s.indent(-3);
    s.step(&mut || stage_prefs::load_prefs(), Some(7))?;
    s.step(&mut || level::load_error_list(), Some(8))?;
    s.step(
        &mut || crate::scene::texture_levels::init_texture_levels(),
        Some(0xA),
    )?;
    s.step(
        &mut || crate::scene::set_up_textures::set_up_textures(),
        Some(0xE),
    )?;
    s.step(&mut || level::build_texture_infos(), Some(0xF))?;
    s.step(
        &mut || crate::race::course_data::locate_current_course(),
        Some(0x13),
    )?;
    s.step(
        &mut || {
            crate::scene::reset_texture_places::reset_texture_places();
            Ok(())
        },
        Some(0x14),
    )?;
    // reloadobjects.
    s.log();
    s.indent(3);
    crate::race::objects::reload_objects()?;
    s.progress(0x1E)?;
    RACE_VARS.bar_span[0].set(0x1E); // dword_623634
    RACE_VARS.bar_span[1].set(0x50); // dword_623638
    s.indent(-3);
    s.step(
        &mut || crate::race::set_all_plates::set_all_plates(),
        Some(0x50),
    )?;
    // loadgamemap.
    s.log();
    s.indent(3);
    crate::race::game_map::load_game_map(
        FILE_NUMBERS.text_ptr(texts::MAP_PATH),
        FILE_NUMBERS.text_ptr(texts::MAP_KEY),
    )?;
    s.progress(0x5A)?;
    s.indent(-3);
    s.step(
        &mut || crate::gfx::build_game_fonts::build_game_fonts_in_game(),
        Some(0x5C),
    )?;
    // init mark, with the log off.
    s.log();
    s.indent(3);
    let enabled =
        crate::logfile::write_line::state::ENABLED.swap(0, std::sync::atomic::Ordering::Relaxed);
    crate::effects::init_mark_step::init_mark_step()?;
    crate::logfile::write_line::state::ENABLED.store(enabled, std::sync::atomic::Ordering::Relaxed);
    s.progress(0x5E)?;
    s.indent(-3);
    // refresh init and the refresh flags.
    s.log();
    s.indent(3);
    level::reset_refresh();
    let flags: u32 = STAGE_PREFS.refresh_flags_a.bits();
    LIGHT_WORK.marks_mask.set_bits((flags & 2) >> 1);
    for car in &CAR_BLOCKS.cars[..8] {
        car.lights_on.set(flags & 1);
    }
    LIGHT_WORK
        .lamps_mask
        .set_bits(STAGE_PREFS.refresh_flags_b.bits() & 1);
    s.indent(-3);
    s.step(
        &mut || {
            stage_prefs::init_skidmarks()?;
            Ok(())
        },
        Some(0x5F),
    )?;
    // render3d init: `scene::allocate_point_records`.
    s.step(
        &mut || {
            crate::scene::allocate_point_records::allocate_point_records_in_game()?;
            Ok(())
        },
        Some(0x60),
    )?;
    s.step(
        &mut || {
            crate::scene::fill_scene_light_tables::fill_scene_light_tables()?;
            Ok(())
        },
        Some(0x62),
    )?;
    s.step(&mut || stage_prefs::load_lights_data(), Some(0x63))?;
    // initmud: mark 0x64 twice, then the callback put away.
    s.log();
    s.indent(3);
    crate::race::mud::init_mud()?;
    s.progress(0x64)?;
    s.progress(0x64)?;
    RACE_VARS.bar_function.set_bits(BAR_DONE);
    s.indent(-3);
    s.step(
        &mut || {
            stage_prefs::get_mat_crash();
            Ok(())
        },
        None,
    )?;
    // store norms; without `dword_64BB18` the checkpoints, the pace notes, the overlay.
    s.log();
    s.indent(3);
    crate::race::norms::store_norms(&mut crate::race::norms::game_normal)?;
    if crate::drive::finish_store::FINISH.no_place.get() == 0 {
        s.indent(-3);
        s.step(&mut || level::init_checkpoints(), None)?;
        s.step(
            &mut || crate::race::init_pace_notes::init_pace_notes(),
            None,
        )?;
        s.log();
        s.indent(3);
        crate::race::init_overlay::init_overlay()?;
    } else {
        s.word += 3;
    }
    use crate::scene::find_object_of_kind::find_object_of_kind;
    let mut marker = find_object_of_kind(0x7A);
    FRAME_SWITCHES.marker_object.set_bits(marker);
    if marker == 0 {
        marker = find_object_of_kind(0x79);
        FRAME_SWITCHES.marker_object.set_bits(marker);
    }
    // dword_719FD0 / D4.
    for cell in &REPLAY_WORK.camera_last_frame {
        cell.set(0);
    }
    s.indent(-3);
    s.log();
    Ok(())
}
