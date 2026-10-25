//! `race::loadlevel_init` (0x44426F, IDA 2026-10-17): the stage's loading proper, step by step,
//! each step's word logged ("LOADLEVEL_INIT : ...") with the log indented round it and the
//! loading bar moved on (`dword_623700` the step's mark, then the callback in `dword_623704`,
//! `sub_4467C7`, called with esi it). The pace notes are `race::init_pace_notes` since
//! 2026-10-19. A step's refusal is the function's (2026-10-23: no longer handed to the step's
//! address).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::game_map::MAP;
use crate::race::numbers_store::NUMBERS;
use crate::race::vars_store::RACE_VARS;
use crate::race::work_store::RACE_WORK;

const EDITOR_RVA: usize = rva(0x621514); // dword_621514
const BAR_RVA: usize = rva(0x4467C7); // sub_4467C7: the loading bar's 3D frame
const BAR_DONE_RVA: usize = rva(0x4467C6); // nullsub_8
const OBJECT_SPAN_RVA: [usize; 2] = [rva(0x623634), rva(0x623638)]; // dword_623634 / 38
const MAP_PATH_RVA: usize = rva(0x60AE12); // "srcmaps\map"
const MAP_KEY_RVA: usize = rva(0x60AE24); // unk_60AE24
const REFRESH_FLAGS_RVA: usize = rva(0x609354); // dword_609354
const REFRESH_FLAGS_2_RVA: usize = rva(0x609350); // dword_609350
const REFRESH_A_RVA: usize = rva(0x62DAD0); // dword_62DAD0
const REFRESH_B_RVA: usize = rva(0x62DACC); // dword_62DACC
const NO_PLACE_RVA: usize = rva(0x64BB18); // dword_64BB18
const MARKER_OBJECT_RVA: usize = rva(0x621508); // dword_621508
const CLEARED_RVA: [usize; 2] = [rva(0x719FD0), rva(0x719FD4)]; // dword_719FD0 / D4

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
    pub indent: u32,
    pub log_enabled: u32,
    pub in_race: u32,
    pub bar: u32,
}

impl LoadVars {
    pub fn game() -> Self {
        Self {
            indent: crate::logfile::write_line::indent_place(), // g_logIndent
            log_enabled: crate::logfile::write_line::enabled_place(), // g_logEnabled
            in_race: NUMBERS.in_race.addr() as u32,             // g_inRace
            bar: game_addr(BAR_RVA) as u32,
        }
    }
}

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) };
}

/// The original's `loadlevel_init`, step by step.
struct Steps {
    v: LoadVars,
    word: usize,
}

impl Steps {
    /// `pusha`; eax the next word; the log; `popa`.
    fn log(&mut self) {
        unsafe {
            crate::logfile::log_write::log_write(
                WORDS[self.word].as_ptr() as usize as *const u8,
                true,
                1,
            )
        };
        self.word += 1;
    }

    /// `pushf`; `g_logIndent` +- 3; `popf`.
    fn indent(&mut self, by: i32) {
        unsafe { put(self.v.indent, get(self.v.indent).wrapping_add(by as u32)) };
    }

    /// `pusha`; `dword_623700` the mark; `call [dword_623704]`; `popa`. The callback is the
    /// loading bar `loadlevel_init` puts there before its first mark (it is put away after
    /// the last).
    fn progress(&mut self, mark: u32) -> Result<(), Refused> {
        unsafe {
            RACE_VARS.bar_share.set_bits(mark);
            if RACE_VARS.bar_function.bits() != self.v.bar {
                return Err(BadInput("the loading bar's callback is not the loading bar").into());
            }
            crate::race::loading_bar_frame::loading_bar_frame()
        }
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
pub unsafe fn loadlevel_init() -> Result<(), Refused> {
    use crate::race::level_init::{self as level, Vars};
    use crate::race::stage_prefs;
    let v = LoadVars::game();
    let mut s = Steps { v, word: 0 };
    unsafe {
        s.log();
        s.indent(3);
        MAP.use_packed.set(1u32);
        MAP.packed_2.set((game_read::<u32>(EDITOR_RVA) == 0) as u32);
        let mut direction: u32 = RACE_VARS.direction.bits();
        if [0x1F, 0x22, 0x51].contains(&RACE_VARS.stage.bits()) {
            direction ^= 1;
        }
        RACE_WORK.stage_direction.set(direction);
        put(v.in_race, 1);
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
        level::allocate_course_memory(&Vars::game(), game_addr(level::COURSE_LIST_RVA) as u32)?;
        s.progress(2)?;
        s.progress(4)?;
        s.indent(-3);
        // load texture info and the course number.
        s.log();
        s.indent(3);
        level::load_texture_info(&Vars::game())?;
        level::set_course_number(RACE_VARS.stage.bits());
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
        s.step(&mut || level::build_texture_infos(&Vars::game()), Some(0xF))?;
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
        game_write(OBJECT_SPAN_RVA[0], 0x1Eu32);
        game_write(OBJECT_SPAN_RVA[1], 0x50u32);
        s.indent(-3);
        s.step(
            &mut || crate::race::set_all_plates::set_all_plates(),
            Some(0x50),
        )?;
        // loadgamemap.
        s.log();
        s.indent(3);
        crate::race::game_map::load_game_map(
            game_addr(MAP_PATH_RVA) as u32,
            game_addr(MAP_KEY_RVA) as u32,
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
        let enabled = get(v.log_enabled);
        put(v.log_enabled, 0);
        crate::marks::init_mark_step::init_mark_step()?;
        put(v.log_enabled, enabled);
        s.progress(0x5E)?;
        s.indent(-3);
        // refresh init and the refresh flags.
        s.log();
        s.indent(3);
        level::reset_refresh(&Vars::game());
        let flags: u32 = game_read(REFRESH_FLAGS_RVA);
        game_write(REFRESH_A_RVA, (flags & 2) >> 1);
        for car in &CAR_BLOCKS.cars[..8] {
            car.lights_on.set(flags & 1);
        }
        game_write(REFRESH_B_RVA, game_read::<u32>(REFRESH_FLAGS_2_RVA) & 1);
        s.indent(-3);
        s.step(
            &mut || {
                stage_prefs::init_skidmarks();
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
                crate::scene::fill_scene_light_tables::fill_scene_light_tables();
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
        RACE_VARS
            .bar_function
            .set_bits(game_addr(BAR_DONE_RVA) as u32);
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
        if game_read::<u32>(NO_PLACE_RVA) == 0 {
            s.indent(-3);
            s.step(&mut || level::init_checkpoints(&Vars::game()), None)?;
            s.step(
                &mut || crate::race::init_pace_notes::init_pace_notes(),
                None,
            )?;
            s.log();
            s.indent(3);
            crate::race::init_overlay::init_overlay(
                &crate::race::init_overlay::OverlayVars::game(),
            )?;
        } else {
            s.word += 3;
        }
        use crate::scene::find_object_of_kind::find_object_of_kind;
        let mut marker = find_object_of_kind(0x7A);
        game_write(MARKER_OBJECT_RVA, marker);
        if marker == 0 {
            marker = find_object_of_kind(0x79);
            game_write(MARKER_OBJECT_RVA, marker);
        }
        for at in CLEARED_RVA {
            game_write(at, 0u32);
        }
        s.indent(-3);
        s.log();
    }
    Ok(())
}
