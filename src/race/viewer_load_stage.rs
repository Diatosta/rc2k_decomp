//! `race::viewer_load_stage` (0x41CDC2): with a replay open (`dword_5BEC88` 1), its stage
//! loaded for the viewer: the front end's 3D tidied (`race::render3d_tidy`,
//! `race::objects_tidy`, `nullsub_6`, `race::release_textures`), the race's screen mode
//! (`race::set_race_screen_mode`), the loading picture (`menu::show_loading_picture`);
//! from the replay's header the game mode, players, cars and options (`dword_719B18` ..
//! into `dword_75ACD8`, `g_onePlayer`, `dword_75ACE8`, `dword_75ACEC`, `dword_75AE20`,
//! `dword_75AF8C`; `dword_75AE28`, `dword_75AF94` 0; `dword_75ACE0` the players; the
//! co-driver's door `byte_719C8C` into `dword_75AE1C` and the cars' block, logged
//! "doornr="); the race's sounds (`race::set_up_race_sound_objects`,
//! `sound::set_up_race_sounds`); the fonts unless there are some (`gfx::build_fonts`,
//! `dword_5BECC4` 1); the header read in (`race::read_replay_header`), the stage
//! (`race::load_stage`), the sounds reset (`race::reset_race_sounds`), the page's
//! palettes (`race::viewer_palettes`), `dword_5BECC0` 1. Then (always) the pending key
//! taken (`input::take_key`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::replay_viewer`, `race::viewer_action` (0x14, after another replay is
//! opened), `race::choose_save_slot`. Its calls through `part` (places in `CALLEES`); in
//! the game their Rust versions, the loading picture and the race sounds' set-up through
//! their hooks (their Rust versions take the game's own calls), nullsub_6 nothing. Long
//! (a stage loads): gated like `race::load_stage`. Shadow mode runs the original alone.
//! Self-test: a copy whose calls go to stand-ins, against the Rust version: no replay
//! open, one open with and without fonts.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::a;
use crate::race::viewer::{
    OPEN, Part, Saved, answer, dispatch_parts, dword, gb, get, put, regs_entry, run_both, test_part,
};
use crate::service::harness::Region;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::viewer_load_stage", rva(0x41CDC2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The calls by place, as IDA shows them.
pub const CALLEES: [usize; 17] = [
    0, 0x4433A7, // 1 race::render3d_tidy
    0x44346B, // 2 race::objects_tidy
    0x44347E, // 3 nullsub_6
    0x443396, // 4 race::release_textures
    0x404830, // 5 race::set_race_screen_mode
    0x47CEA8, // 6 menu::show_loading_picture
    0x401427, // 7 logfile::log_write_l1
    0x401476, // 8 logfile::log_int_l1_nl
    0x415C93, // 9 race::set_up_race_sound_objects
    0x40F02E, // 10 sound::set_up_race_sounds
    0x415563, // 11 gfx::build_fonts
    0x460428, // 12 race::read_replay_header
    0x444040, // 13 race::load_stage
    0x40D378, // 14 race::reset_race_sounds
    0x41D8BF, // 15 race::viewer_palettes
    0x407F95, // 16 input::take_key
];
const CALLS: [usize; 16] = [
    0xE, 0x13, 0x18, 0x1D, 0x22, 0x27, 0x9E, 0xA5, 0xAB, 0xB0, 0xBE, 0xCD, 0xD2, 0xD7, 0xDC, 0xEB,
];

const LOADED: usize = 0x5BECC0; // dword_5BECC0: the viewer's stage is loaded
pub const FONTS_BUILT: usize = 0x5BECC4; // dword_5BECC4: the viewer built the fonts
const FONT_PALETTES: usize = 0x51DEB0; // g_fontPalettes
const DOOR: usize = 0x719C8C; // byte_719C8C
const DOOR_TEXT: usize = 0x4DB12C; // "doornr="
/// (to, from): the header's settings.
const SETTINGS: [(usize, usize); 6] = [
    (0x75ACD8, 0x719B18),
    (0x514F78, 0x719B34), // g_onePlayer
    (0x75ACE8, 0x719B24),
    (0x75ACEC, 0x719B28),
    (0x75AE20, 0x719B38),
    (0x75AF8C, 0x719B3C),
];

/// The Rust version: the registers it leaves (the entry's).
pub fn viewer_load_stage(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if get(OPEN) == 1 {
        for place in 1..=6 {
            part(place, &mut r)?;
        }
        for (to, from) in SETTINGS {
            r.eax = get(from);
            put(to, r.eax);
        }
        put(0x75AE28, 0);
        put(0x75AF94, 0);
        r.eax = get(0x514F78);
        put(0x75ACE0, r.eax);
        r.eax = gb(DOOR) as u32;
        put(0x75AE1C, r.eax);
        put(0x70FE94, r.eax); // g_carBlockStore.gap_a28 + 0x80
        let mut s = Regs {
            eax: a(DOOR_TEXT),
            ..r
        };
        part(7, &mut s)?;
        let mut s = r;
        part(8, &mut s)?;
        part(9, &mut r)?;
        part(10, &mut r)?;
        if get(FONT_PALETTES) == 0 {
            part(11, &mut r)?;
            put(FONTS_BUILT, 1);
        }
        for place in 12..=15 {
            part(place, &mut r)?;
        }
        put(LOADED, 1);
    }
    part(16, &mut r)?;
    Ok(*entry)
}

/// In the game: the Rust versions; the loading picture and the race sounds' set-up
/// through their hooks.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::{race_end, race_shell, sound_tables, stage_loading};
    unsafe {
        match place {
            1 | 2 => race_end::menu_part(place, r)?,
            3 => {}
            4 => race_end::menu_part(3, r)?,
            5 => race_shell::set_race_screen_mode(r, &mut race_shell::game_screen_mode),
            6 | 10 => crate::calls::call(rva(CALLEES[place as usize]), r),
            7 => crate::logfile::log_write::log_write(r.eax as *const u8, false, 1),
            8 => crate::logfile::log_int::log_int(r.eax as i32, true, 1),
            9 => {
                sound_tables::set_up_race_sound_objects(
                    r,
                    &sound_tables::ObjectVars::game(),
                    &mut sound_tables::object_part,
                )?;
            }
            11 => {
                crate::gfx::build_fonts::build_fonts(&{ *r }, &mut crate::gfx::glyph_tables::Game)?
            }
            12 => *r = crate::race::read_replay_header::read_replay_header(r, &mut log_part)?,
            13 => {
                let indent = a(0x4E00A0); // g_logIndent
                *r = stage_loading::load_stage(r, indent, &mut stage_loading::game_part)?;
                r.set_carry(false);
            }
            14 => {
                race_end::reset_race_sounds(
                    r,
                    &race_end::SoundVars::game(),
                    &mut race_end::sound_part,
                )?;
            }
            15 => crate::race::viewer_palettes::viewer_palettes(),
            _ => take_key(r),
        }
    }
    Ok(())
}

/// `input::take_key`'s Rust version into the registers (al, ZF).
pub fn take_key(r: &mut Regs) {
    let key = crate::input::take_key::take_key();
    r.eax = r.eax & !0xFF | key as u32;
    r.set_zero(key == 0);
    r.set_carry(false);
}

/// The log calls of `race::read_replay_header` in the game.
pub fn log_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::race::read_replay_header::game_part(place, r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        viewer_load_stage(r, &mut game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| viewer_load_stage(r, &mut game_part))
}

fn regions() -> Vec<Region> {
    let mut all: Vec<Region> = SETTINGS.iter().map(|&(to, _)| dword(to)).collect();
    for at in [
        0x75AE28usize,
        0x75AF94,
        0x75ACE0,
        0x75AE1C,
        0x70FE94,
        LOADED,
        FONTS_BUILT,
    ] {
        all.push(dword(at));
    }
    all
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places: &'static [(usize, u32)] = Box::leak(
        (1..17u32)
            .map(|p| (CALLEES[p as usize], p))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    // pusha; cmp dword_5BEC88, 1
    let copy = match crate::race::viewer::copy(
        0x41CDC2,
        0xF2,
        &[0x60, 0x83, 0x3D, 0x88, 0xEC, 0x5B, 0x00, 0x01],
        &CALLS,
        places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::race::viewer::quiet();
    let mut all = regions();
    all.extend([
        dword(OPEN),
        dword(FONT_PALETTES),
        crate::race::viewer::region(DOOR, 1),
    ]);
    all.extend(SETTINGS.iter().map(|&(_, from)| dword(from)));
    let saved = Saved::new(&all);
    for (open, fonts) in [(0u32, 1u32), (1, 1), (1, 0)] {
        run_both(
            &mut t,
            &format!("open {open}, fonts {fonts}"),
            copy.addr(),
            Regs::SENTINEL,
            &regions(),
            0,
            &|| {
                put(OPEN, open);
                put(FONT_PALETTES, fonts);
                for (n, &(to, from)) in SETTINGS.iter().enumerate() {
                    put(from, 0x10 + n as u32);
                    put(to, 0xAAAA_AAAA);
                }
                crate::race::viewer::pb(DOOR, 0x83);
                put(LOADED, 0);
                put(FONTS_BUILT, 0);
                answer(crate::race::viewer::clear_carry());
            },
            &|r| viewer_load_stage(r, &mut test_part),
        );
    }
    crate::race::viewer::quiet();
    drop(saved);
    t.finish()
}
