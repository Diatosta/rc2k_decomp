//! `race::loadlevel_init` (0x44426F, IDA 2026-10-17): the stage's loading proper, step by
//! step, each step's word logged ("LOADLEVEL_INIT : ...") with the log indented round it
//! and the loading bar moved on (`dword_623700` the step's mark, then the callback in
//! `dword_623704`, `sub_4467C7`, called with esi it). Every step is Rust but two, which
//! are called through the game (`call_game`, at their hooks): init mark (`sub_456EA7`,
//! the marks) and the loading bar's callback (`nullsub_8` after the loading: nothing). The
//! pace notes are `race::init_pace_notes` since 2026-10-19. A step that refuses is run in the game
//! through its own hook (`or_in_game`), so the function itself refuses nothing.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::service::harness::{Region, at};

/// A step's calls by callee (`CALLEES`' index; 28 the log, 29 the loading bar).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The callees in their first call's order, as IDA shows them.
pub const CALLEES: [usize; 28] = [
    0x4433C9, // 0 scene::set_up_object_memory
    0x41F8C4, // 1 race::allocate_course_memory
    0x4398FA, // 2 race::load_texture_info
    0x43944F, // 3 race::set_course_number
    0x445698, // 4 race::load_prefs
    0x439A92, // 5 race::load_error_list
    0x424001, // 6 scene::init_texture_levels
    0x443390, // 7 sub_443390 (texturesalloc: `scene::set_up_textures`)
    0x4393A0, // 8 race::build_texture_infos
    0x4394CB, // 9 race::locate_current_course
    0x423FE1, // 10 scene::reset_texture_places
    0x425B2F, // 11 race::reload_objects
    0x42518B, // 12 race::set_all_plates
    0x41B480, // 13 race::load_game_map
    0x41B178, // 14 gfx::build_game_fonts
    0x456EA7, // 15 sub_456EA7 (init mark: the game's)
    0x45C53B, // 16 race::reset_refresh
    0x45857D, // 17 race::init_skidmarks
    0x4433A1, // 18 race::render3d_init
    0x4433AD, // 19 scene::fill_scene_light_tables
    0x438AE5, // 20 race::load_lights_data
    0x449DD6, // 21 race::init_mud
    0x43D43E, // 22 race::get_mat_crash
    0x421F02, // 23 race::store_norms
    0x45A98F, // 24 race::init_checkpoints
    0x441A66, // 25 race::init_pace_notes
    0x45A9DE, // 26 race::init_overlay
    0x422298, // 27 scene::find_object_of_kind
];
pub const LOG: u32 = 28;
pub const PROGRESS: u32 = 29;
pub const LOG_RVA: usize = 0x40142B; // logfile::log_write_l1_nl

const PACKED_RVA: usize = rva(0x5BE9B5); // g_usePackedMaps
const PACKED_2_RVA: usize = rva(0x5BE9B9); // dword_5BE9B9
const EDITOR_RVA: usize = rva(0x621514); // dword_621514
const DIRECTION_IN_RVA: usize = rva(0x623854); // dword_623854
const DIRECTION_RVA: usize = rva(0x71B400); // g_stageDirection
const STAGE_RVA: usize = rva(0x623850); // g_stageNumber
pub const BAR_MARK_RVA: usize = rva(0x623700); // dword_623700
pub const BAR_CALLBACK_RVA: usize = rva(0x623704); // dword_623704
const BAR_RVA: usize = rva(0x4467C7); // sub_4467C7: the loading bar's 3D frame
const BAR_DONE_RVA: usize = rva(0x4467C6); // nullsub_8
const OBJECT_SPAN_RVA: [usize; 2] = [rva(0x623634), rva(0x623638)]; // dword_623634 / 38
const MAP_PATH_RVA: usize = rva(0x60AE12); // "srcmaps\map"
const MAP_KEY_RVA: usize = rva(0x60AE24); // unk_60AE24
const REFRESH_FLAGS_RVA: usize = rva(0x609354); // dword_609354
const REFRESH_FLAGS_2_RVA: usize = rva(0x609350); // dword_609350
const REFRESH_A_RVA: usize = rva(0x62DAD0); // dword_62DAD0
const REFRESH_B_RVA: usize = rva(0x62DACC); // dword_62DACC
/// Each car block's +0x84 (dword_70F470 ..).
const BLOCK_84: [usize; 8] = [
    0x70F470, 0x710450, 0x711430, 0x712410, 0x7133F0, 0x7143D0, 0x7153B0, 0x716390,
];
const NO_PLACE_RVA: usize = rva(0x64BB18); // dword_64BB18
const MARKER_OBJECT_RVA: usize = rva(0x621508); // dword_621508
const CLEARED_RVA: [usize; 2] = [rva(0x719FD0), rva(0x719FD4)]; // dword_719FD0 / D4

/// The step words ("LOADLEVEL_INIT : start" .. "finis"), in order.
const WORDS: [usize; 28] = [
    0x4DBE6E, 0x4DBE85, 0x4DBEA3, 0x4DBEC4, 0x4DBEE7, 0x4DBF03, 0x4DBF23, 0x4DBF45, 0x4DBF64,
    0x4DBF87, 0x4DBFA7, 0x4DBFC7, 0x4DBFE6, 0x4DC004, 0x4DC021, 0x4DC040, 0x4DC05B, 0x4DC079,
    0x4DC099, 0x4DC0B8, 0x4DC0DD, 0x4DC106, 0x4DC128, 0x4DC145, 0x4DC167, 0x4DC188, 0x4DC1A3,
    0x4DC1C0,
];

/// Where the variables the self-test keeps apart are: the log's indent and switch,
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
            indent: game_addr(rva(0x4E00A0)) as u32,      // g_logIndent
            log_enabled: game_addr(rva(0x4E0094)) as u32, // g_logEnabled
            in_race: game_addr(rva(0x5C5CEC)) as u32,     // g_inRace
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
struct Steps<'a, 'b> {
    g: Regs,
    v: LoadVars,
    part: Part<'a>,
    word: usize,
    _b: std::marker::PhantomData<&'b ()>,
}

impl Steps<'_, '_> {
    /// `pusha`; eax the next word; the log; `popa`.
    fn log(&mut self) -> Result<(), Refused> {
        let mut r = Regs {
            eax: game_addr(rva(WORDS[self.word])) as u32,
            ..self.g
        };
        self.word += 1;
        (self.part)(LOG, &mut r)
    }

    /// `pushf`; `g_logIndent` +- 3; `popf`.
    fn indent(&mut self, by: i32) {
        unsafe { put(self.v.indent, get(self.v.indent).wrapping_add(by as u32)) };
    }

    /// `pusha`; `dword_623700` the mark; esi `dword_623704`; its call; `popa`.
    fn progress(&mut self, mark: u32) -> Result<(), Refused> {
        unsafe {
            game_write(BAR_MARK_RVA, mark);
            let mut r = Regs {
                esi: game_read(BAR_CALLBACK_RVA),
                ..self.g
            };
            (self.part)(PROGRESS, &mut r)
        }
    }

    fn call(&mut self, callee: u32) -> Result<(), Refused> {
        (self.part)(callee, &mut self.g)
    }

    /// A step's usual frame: the word, the indent in, the call, the mark, the indent out.
    fn step(&mut self, callee: u32, mark: Option<u32>) -> Result<(), Refused> {
        self.log()?;
        self.indent(3);
        self.call(callee)?;
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
/// `dword_719FD0`/`D4` 0; "finis". Leaves the registers as the last call.
pub unsafe fn loadlevel_init(entry: &Regs, v: LoadVars, part: Part) -> Result<Regs, Refused> {
    let mut s = Steps {
        g: *entry,
        v,
        part,
        word: 0,
        _b: std::marker::PhantomData,
    };
    unsafe {
        s.log()?;
        s.indent(3);
        game_write(PACKED_RVA, 1u32);
        game_write(PACKED_2_RVA, (game_read::<u32>(EDITOR_RVA) == 0) as u32);
        s.g.eax = game_read(DIRECTION_IN_RVA);
        if [0x1F, 0x22, 0x51].contains(&game_read::<u32>(STAGE_RVA)) {
            s.g.eax ^= 1;
        }
        game_write(DIRECTION_RVA, s.g.eax);
        put(v.in_race, 1);
        game_write(BAR_CALLBACK_RVA, v.bar);
        s.progress(0)?;
        s.indent(-3);
        s.step(0, None)?;
        // alloc coursemem: marks 2 and 4 (the `cmp dword_621514, 0 / jz $+2` between
        // them changes nothing).
        s.log()?;
        s.indent(3);
        s.call(1)?;
        s.progress(2)?;
        s.progress(4)?;
        s.indent(-3);
        // load texture info and the course number.
        s.log()?;
        s.indent(3);
        s.call(2)?;
        s.g.eax = game_read(STAGE_RVA);
        s.call(3)?;
        s.progress(6)?;
        s.indent(-3);
        for (callee, mark) in [
            (4, 7),
            (5, 8),
            (6, 0xA),
            (7, 0xE),
            (8, 0xF),
            (9, 0x13),
            (10, 0x14),
        ] {
            s.step(callee, Some(mark))?;
        }
        // reloadobjects.
        s.log()?;
        s.indent(3);
        s.call(11)?;
        s.progress(0x1E)?;
        game_write(OBJECT_SPAN_RVA[0], 0x1Eu32);
        game_write(OBJECT_SPAN_RVA[1], 0x50u32);
        s.indent(-3);
        s.step(12, Some(0x50))?;
        // loadgamemap.
        s.log()?;
        s.indent(3);
        s.g.eax = game_addr(MAP_PATH_RVA) as u32;
        s.g.ebx = game_addr(MAP_KEY_RVA) as u32;
        s.call(13)?;
        s.progress(0x5A)?;
        s.indent(-3);
        s.step(14, Some(0x5C))?;
        // init mark, with the log off.
        s.log()?;
        s.indent(3);
        let enabled = get(v.log_enabled);
        put(v.log_enabled, 0);
        s.call(15)?;
        put(v.log_enabled, enabled);
        s.progress(0x5E)?;
        s.indent(-3);
        // refresh init and the refresh flags.
        s.log()?;
        s.indent(3);
        s.call(16)?;
        let flags: u32 = game_read(REFRESH_FLAGS_RVA);
        game_write(REFRESH_A_RVA, (flags & 2) >> 1);
        for b in BLOCK_84 {
            game_write(rva(b), flags & 1);
        }
        s.g.eax = game_read::<u32>(REFRESH_FLAGS_2_RVA) & 1;
        game_write(REFRESH_B_RVA, s.g.eax);
        s.indent(-3);
        for (callee, mark) in [(17, 0x5F), (18, 0x60), (19, 0x62), (20, 0x63)] {
            s.step(callee, Some(mark))?;
        }
        // initmud: mark 0x64 twice, then the callback put away.
        s.log()?;
        s.indent(3);
        s.call(21)?;
        s.progress(0x64)?;
        s.progress(0x64)?;
        game_write(BAR_CALLBACK_RVA, game_addr(BAR_DONE_RVA) as u32);
        s.indent(-3);
        s.step(22, None)?;
        // store norms; without `dword_64BB18` the checkpoints, the pace notes, the overlay.
        s.log()?;
        s.indent(3);
        s.call(23)?;
        if game_read::<u32>(NO_PLACE_RVA) == 0 {
            s.indent(-3);
            s.step(24, None)?;
            s.step(25, None)?;
            s.log()?;
            s.indent(3);
            s.call(26)?;
        } else {
            s.word += 3;
        }
        s.g.eax = 0x7A;
        s.call(27)?;
        game_write(MARKER_OBJECT_RVA, s.g.eax);
        if s.g.eax == 0 {
            s.g.eax = 0x79;
            s.call(27)?;
            game_write(MARKER_OBJECT_RVA, s.g.eax);
        }
        for at in CLEARED_RVA {
            game_write(at, 0u32);
        }
        s.indent(-3);
        s.log()?;
    }
    Ok(s.g)
}

/// A step's Rust version, or, when it refuses a case it has not worked out, the step
/// called in the game through its hook (registers as at entry): that hook refuses again,
/// logs it and runs the step's original. So a refusal mid-way costs one step, not the
/// whole function run again over the steps already done.
pub fn or_in_game(
    address: usize,
    r: &mut Regs,
    rust: impl FnOnce(&mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let entry = *r;
    match rust(r) {
        Err(Refused::BadInput(_)) => {
            *r = entry;
            crate::calls::call(rva(address), r);
            Ok(())
        }
        other => other,
    }
}

/// The calls in the game: the Rust versions (each through `or_in_game`); init mark and the
/// loading bar through the game.
pub fn game_part(callee: u32, r: &mut Regs) -> Result<(), Refused> {
    match callee {
        0..28 => or_in_game(CALLEES[callee as usize], r, |r| step_part(callee, r)),
        _ => step_part(callee, r),
    }
}

fn step_part(callee: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::level_init::Vars;
    unsafe {
        match callee {
            0 => crate::scene::set_up_object_memory::set_up_object_memory_in_game(r)?,
            1 => {
                *r = crate::race::level_init::allocate_course_memory(
                    r,
                    &Vars::game(),
                    game_addr(crate::race::level_init::COURSE_LIST_RVA) as u32,
                    &mut crate::race::level_init::game_part,
                )?
            }
            2 => {
                *r = crate::race::level_init::load_texture_info(
                    r,
                    &Vars::game(),
                    &mut crate::race::level_init::game_part,
                )?
            }
            3 => crate::race::level_init::set_course_number(r.eax),
            4 => {
                *r = crate::race::stage_prefs::load_prefs(
                    r,
                    &mut crate::race::level_init::game_part,
                )?
            }
            5 => crate::race::level_init::load_error_list(
                r,
                &mut crate::race::level_init::game_part,
            )?,
            6 => {
                *r = crate::scene::texture_levels::init_texture_levels(
                    r,
                    &mut crate::scene::texture_levels::game_part,
                )?
            }
            7 => {
                use crate::scene::build_texture_catalogue::build_texture_catalogue_in_game;
                crate::scene::set_up_textures::set_up_textures(
                    r,
                    &mut |_| build_texture_catalogue_in_game(),
                    &mut |r| {
                        crate::calls::call(rva(0x4051ED), r); // gfx::allocate_textures
                        !r.carry()
                    },
                )?;
                r.set_carry(false);
            }
            8 => *r = crate::race::level_init::build_texture_infos(r, &Vars::game())?,
            9 => {
                *r = crate::race::course_data::locate_current_course(
                    r,
                    &mut crate::race::course_data::locate_part,
                )?
            }
            10 => crate::scene::reset_texture_places::reset_texture_places().apply(r),
            11 => {
                *r = crate::race::objects::reload_objects(r, &mut crate::race::objects::game_part)?
            }
            12 => {
                *r = crate::race::set_all_plates::set_all_plates(
                    r,
                    &mut crate::race::set_all_plates::game_part,
                )?
            }
            13 => {
                let g =
                    crate::race::game_map::load_game_map(r, &mut crate::race::game_map::game_part)?;
                r.eax = g.eax;
            }
            14 => {
                // `PALETTE_B_RVA` of `gfx/build_game_fonts.rs`: the third font's palette.
                let pixels_b = game_addr(rva(0x51E2E4) + 2 * 0x20 + 0x14) as *const u32;
                crate::gfx::build_game_fonts::build_game_fonts(
                    r,
                    &mut crate::gfx::glyph_tables::Game,
                    pixels_b,
                )?;
            }
            15 => crate::calls::call(rva(CALLEES[callee as usize]), r),
            25 => crate::race::init_pace_notes::init_pace_notes(r)?,
            16 => crate::race::level_init::reset_refresh(&Vars::game()),
            17 => *r = crate::race::stage_prefs::init_skidmarks(r),
            18 => crate::race::level_init::game_part(2, r)?,
            19 => crate::scene::fill_scene_light_tables::fill_scene_light_tables().apply(r),
            20 => {
                *r = crate::race::stage_prefs::load_lights_data(
                    r,
                    &mut crate::race::level_init::game_part,
                )?
            }
            21 => *r = crate::race::mud::init_mud(r, &mut crate::race::mud::game_part)?,
            22 => *r = crate::race::stage_prefs::get_mat_crash(r),
            23 => {
                let g = crate::race::norms::store_norms(r, &mut crate::race::norms::game_normal)?;
                r.ebx = g.ebx;
            }
            24 => *r = crate::race::level_init::init_checkpoints(r, &Vars::game())?,
            26 => {
                *r = crate::race::init_overlay::init_overlay(
                    r,
                    &crate::race::init_overlay::OverlayVars::game(),
                )?
            }
            27 => r.eax = crate::scene::find_object_of_kind::find_object_of_kind(r.eax),
            LOG => crate::logfile::log_write::log_write(r.eax as *const u8, true, 1),
            // The loading bar's callback; `nullsub_8` (after the loading) does nothing.
            _ if r.esi == game_addr(BAR_DONE_RVA) as u32 => {}
            _ => crate::menu::table_calls::call_any(r.esi, r),
        }
    }
    Ok(())
}

// ---- for the self-test ----

/// The stand-in place of a callee (24 stand-ins for 30 kinds of call: the last six
/// callees share the first six's places; their order still shows), the log 22, the
/// loading bar 23.
pub fn place_of(callee: u32) -> u32 {
    match callee {
        LOG => 22,
        PROGRESS => 23,
        c if c < 22 => c,
        c => c - 22,
    }
}

/// The copy's calls redirected by target (the loading bar's is a `call esi`, by the
/// patched callback).
pub fn redirect(target: usize) -> Option<usize> {
    if target == rva(LOG_RVA) {
        return Some(crate::install::stand_ins::stand_in(22));
    }
    CALLEES
        .iter()
        .position(|&k| rva(k) == target)
        .map(|n| crate::install::stand_ins::stand_in(place_of(n as u32)))
}

/// What the function writes or reads that the test keeps.
pub fn regions() -> Vec<Region> {
    let mut r: Vec<Region> = [
        PACKED_RVA,
        PACKED_2_RVA,
        EDITOR_RVA,
        DIRECTION_IN_RVA,
        DIRECTION_RVA,
        STAGE_RVA,
        BAR_MARK_RVA,
        BAR_CALLBACK_RVA,
        OBJECT_SPAN_RVA[0],
        OBJECT_SPAN_RVA[1],
        REFRESH_FLAGS_RVA,
        REFRESH_FLAGS_2_RVA,
        REFRESH_A_RVA,
        REFRESH_B_RVA,
        NO_PLACE_RVA,
        MARKER_OBJECT_RVA,
        CLEARED_RVA[0],
        CLEARED_RVA[1],
    ]
    .map(at)
    .to_vec();
    r.extend(BLOCK_84.map(|b| at(rva(b))));
    r
}

/// A case's inputs.
pub fn prepare(editor: u32, direction: u32, stage: u32, flags: [u32; 2], no_place: u32) {
    unsafe {
        for &(a, len) in &regions() {
            std::ptr::write_bytes(a as *mut u8, 0x5A, len);
        }
        game_write(EDITOR_RVA, editor);
        game_write(DIRECTION_IN_RVA, direction);
        game_write(STAGE_RVA, stage);
        game_write(REFRESH_FLAGS_RVA, flags[0]);
        game_write(REFRESH_FLAGS_2_RVA, flags[1]);
        game_write(NO_PLACE_RVA, no_place);
    }
}
