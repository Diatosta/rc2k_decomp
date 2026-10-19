//! `scene::set_up_front_end_scene` (0x474E04): the front end's 3D set-up: which
//! textures and objects the menus need, and everything loaded.
//!
//! In:  nothing (the registers go on to the first call as they come)
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! One list of stores and 50 calls, in this order:
//! - the loading bar's range `dword_623634` = 0, `dword_623638` = 100; the loading
//!   picture; `scene::reset_texture_tables`;
//! - the textures needed (`NEEDS`): level 0 and a tag for the menus' own 42 system and
//!   8 video textures from 0x258, the carousel pictures' 0x110 and 0x20 from 0x2BC, and
//!   texture 0x87. (Each store of the level is there twice, the second under
//!   `dword_5CDADC`: the same value either way.)
//! - `scene::set_up_textures`, `scene::set_up_object_memory`, `scene::clear_slot_kinds`;
//! - the scene's colours and sizes (`FIRST_STORES`), `scene::clear_fog`, `dword_621480`
//!   = 1, `scene::fill_light_tables`;
//! - the slots' kinds: slot 0 kind 1, slots 1 .. 0x28 kinds 0x259 .. 0x280, slots 0x29
//!   .. 0x2C kinds 0x28B .. 0x28E, slots 0x2D .. 0x4E the 0x22 known kinds
//!   (`scene::known_kind`);
//! - `scene::load_front_end_objects` and nine lines in the game's log with the points
//!   and faces loaded (the stub `sub_4433DA`, inlined); `scene::reset_texture_places`;
//! - the kinds' pictures (`g_kindPictures`): kind 0x87 picture 0x82A, kinds 0x258 ..
//!   0x271 the 26 of `MENU_PICTURES`, and eight in a row from kind 0x2BC + 8 * place for
//!   each of 33 known kinds (`scene::known_kind_picture` of places 0 .. 0x1D, 0x1F,
//!   0x20, 0x21, and the seven numbers after it; place 0x1E's eight are left alone);
//! - the loading bar's function `dword_623704` = `sub_4467C7`,
//!   `scene::reload_system_textures`, `scene::allocate_point_records`;
//! - the colours again, `scene::fill_scene_light_tables`, `scene::set_view_matrices`,
//!   `scene::reload_slot_textures`;
//! - the colours a third time with `dword_6092BC` = 0x7F, `dword_621480` = 1,
//!   `scene::fill_light_tables`, `dword_6238AC` = 1.
//!
//! Thirteen callers (the main menu's loop, the films, the ends of the events).
//! The loading picture, the textures' set-up, the objects' loading and the system
//! textures' reloading are called at the game's addresses (their hooks choose the
//! version); the rest are the Rust versions.
//! A refusal by one of its parts comes with the work half done; the original then
//! starts from the top.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose calls (82 made) note themselves and everything written
//! by then.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::allocate_point_records::allocate_point_records_in_game;
use crate::scene::clear_fog::clear_fog;
use crate::scene::clear_slot_kinds::clear_slot_kinds;
use crate::scene::fill_light_tables::fill_light_tables;
use crate::scene::fill_scene_light_tables::fill_scene_light_tables;
use crate::scene::known_kind::known_kind;
use crate::scene::known_kind_picture::known_kind_picture;
use crate::scene::reload_slot_textures::reload_slot_textures_in_game;
use crate::scene::reset_texture_places::reset_texture_places;
use crate::scene::reset_texture_tables::reset_texture_tables;
use crate::scene::set_up_object_memory::set_up_object_memory_in_game;
use crate::scene::texture_needs::{SYSTEM_LEVELS_RVA, TAGS_RVA, VIDEO_LEVELS_RVA};
use crate::scene::view_matrix::set_view_matrices;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::set_up_front_end_scene",
    rva(0x474E04),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const BAR_FROM_RVA: usize = rva(0x623634); // dword_623634, dword_623638
const BAR_FUNCTION_RVA: usize = rva(0x623704); // dword_623704
const SHOW_BAR_RVA: usize = rva(0x4467C7); // sub_4467C7
const LIT_COLOUR_RVA: usize = rva(0x6092C0); // dword_6092C0, three dwords
const DARK_COLOUR_RVA: usize = rva(0x6092B4); // dword_6092B4, three dwords
const OTHER_WAY_RVA: usize = rva(0x621480); // dword_621480
const SET_UP_RVA: usize = rva(0x6238AC); // dword_6238AC
const SLOT_KINDS_RVA: usize = rva(0x5CCE7C); // g_slotKinds
const KIND_PICTURES_RVA: usize = rva(0x5D7968); // g_kindPictures
const TEXTURES: usize = 0x400;

/// The textures needed: how many, the first, the levels' table, the tag.
const NEEDS: [(usize, usize, usize, u32); 6] = [
    (0x2A, 0x258, SYSTEM_LEVELS_RVA, 0x63),
    (8, 0x258, VIDEO_LEVELS_RVA, 0x63),
    (0x110, 0x2BC, SYSTEM_LEVELS_RVA, 0x51),
    (0x20, 0x2BC, VIDEO_LEVELS_RVA, 0x51),
    (1, 0x87, SYSTEM_LEVELS_RVA, 0x63),
    (1, 0x87, VIDEO_LEVELS_RVA, 0x63),
];

/// The scene's settings stored before the fog is cleared, after the two colours.
const FIRST_STORES: [(usize, u32); 10] = [
    (rva(0x609300), 0x100),
    (rva(0x609304), 0x100),
    (rva(0x609308), 0),
    (rva(0x609320), 0),
    (rva(0x6092CC), 0),
    (rva(0x6092D0), 0),
    (rva(0x6092D4), 0),
    (rva(0x609310), 0),
    (rva(0x609358), 0),
    (rva(0x60930C), 0),
];

/// The pictures of kinds 0x258 on.
const MENU_PICTURES: [u32; 26] = [
    0x7D1, 0x7D2, 0x7D3, 0x7D5, 0x7D7, 0x7D8, 0x7D9, 0x7DA, 0x7DB, 0x7DC, 0x7DE, 0x7DF, 0x7E1,
    0x7E2, 0x7E3, 0x7E6, 0x7E8, 0x7EE, 0x7EF, 0x7F0, 0x7F3, 0x7F4, 0x7F5, 0x7F6, 0x7F7, 0x7F8,
];

/// The known kinds whose pictures the carousels show: every place but 0x1E.
fn picture_places() -> impl Iterator<Item = u32> {
    (0..0x22).filter(|&place| place != 0x1E)
}

/// What it calls, in the order of their first calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    ShowLoadingPicture,
    ResetTextureTables,
    SetUpTextures,
    SetUpObjectMemory,
    ClearSlotKinds,
    ClearFog,
    FillLightTables,
    /// eax = the place -> eax = the kind; nothing else may change.
    KnownKind,
    /// `sub_4433DA`: the objects loaded and the log's lines.
    LoadObjects,
    ResetTexturePlaces,
    /// eax = the place -> eax = the picture; nothing else may change.
    KnownKindPicture,
    ReloadSystemTextures,
    AllocatePointRecords,
    FillSceneLightTables,
    SetViewMatrices,
    ReloadSlotTextures,
}

/// The callees' addresses as the function's calls have them, by `Call`.
const CALLEES: [usize; 16] = [
    rva(0x47CEA8),
    rva(0x44338A),
    rva(0x443390),
    rva(0x4433C9),
    rva(0x4433D4),
    rva(0x44FC19),
    rva(0x44F0AB),
    rva(0x41F469),
    rva(0x4433DA),
    rva(0x443472),
    rva(0x41F44D),
    rva(0x443478),
    rva(0x4433A1),
    rva(0x4433AD),
    rva(0x438FBF),
    rva(0x425669),
];

// ---- Rust version ----

unsafe fn colours(dark_third: u32) {
    unsafe {
        game_write(LIT_COLOUR_RVA, [0xFFu32; 3]);
        game_write(DARK_COLOUR_RVA, [0u32, 0, dark_third]);
    }
}

/// `step`: what stands for each call; it is entered with the registers as they are by
/// then.
pub unsafe fn set_up_front_end_scene(
    r: &mut Regs,
    step: &mut dyn FnMut(Call, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let entry = *r;
        game_write(BAR_FROM_RVA, [0u32, 100]);
        step(Call::ShowLoadingPicture, r)?;
        step(Call::ResetTextureTables, r)?;
        for (count, first, levels, tag) in NEEDS {
            for texture in first..first + count {
                game_write(levels + texture * 4, 0u32);
                game_write(TAGS_RVA + texture * 4, tag);
            }
            r.ecx = 0;
            r.ebx = (first + count) as u32;
        }
        step(Call::SetUpTextures, r)?;
        step(Call::SetUpObjectMemory, r)?;
        step(Call::ClearSlotKinds, r)?;
        colours(0);
        for (place, value) in FIRST_STORES {
            game_write(place, value);
        }
        step(Call::ClearFog, r)?;
        game_write(OTHER_WAY_RVA, 1u32);
        step(Call::FillLightTables, r)?;

        game_write(SLOT_KINDS_RVA, 1u32);
        for slot in 1..0x29 {
            game_write(SLOT_KINDS_RVA + slot * 4, 0x258 + slot as u32);
        }
        for slot in 0x29..0x2D {
            game_write(SLOT_KINDS_RVA + slot * 4, 0x262 + slot as u32);
        }
        r.edi = 0;
        while r.edi != 0x22 {
            r.eax = r.edi;
            step(Call::KnownKind, r)?;
            game_write(SLOT_KINDS_RVA + (0x2D + r.edi as usize) * 4, r.eax);
            r.edi += 1;
        }
        step(Call::LoadObjects, r)?;
        step(Call::ResetTexturePlaces, r)?;

        game_write(KIND_PICTURES_RVA + 0x87 * 4, 0x82Au32);
        game_write(KIND_PICTURES_RVA + 0x258 * 4, MENU_PICTURES);
        r.edi = game_addr(KIND_PICTURES_RVA + 0x2BC * 4) as u32;
        for place in picture_places() {
            r.eax = place;
            step(Call::KnownKindPicture, r)?;
            for k in 0..8 {
                game_write(
                    KIND_PICTURES_RVA + (0x2BC + place as usize * 8 + k) * 4,
                    r.eax,
                );
                r.eax = r.eax.wrapping_add(1);
            }
        }
        game_write(BAR_FUNCTION_RVA, game_addr(SHOW_BAR_RVA) as u32);
        step(Call::ReloadSystemTextures, r)?;
        step(Call::AllocatePointRecords, r)?;
        colours(0);
        step(Call::FillSceneLightTables, r)?;
        step(Call::SetViewMatrices, r)?;
        step(Call::ReloadSlotTextures, r)?;
        colours(0x7F);
        game_write(OTHER_WAY_RVA, 1u32);
        step(Call::FillLightTables, r)?;
        game_write(SET_UP_RVA, 1u32);
        *r = entry;
        Ok(())
    }
}

/// With the Rust versions of what it calls, or their hooks.
pub unsafe fn set_up_front_end_scene_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        set_up_front_end_scene(r, &mut |call, r| {
            match call {
                Call::ShowLoadingPicture => call_game(r, game_addr(rva(0x47CEA8))),
                Call::ResetTextureTables => reset_texture_tables().apply(r),
                Call::SetUpTextures => call_game(r, game_addr(rva(0x44E7A5))),
                Call::SetUpObjectMemory => set_up_object_memory_in_game(r)?,
                Call::ClearSlotKinds => clear_slot_kinds().apply(r),
                Call::ClearFog => clear_fog().apply(r),
                Call::FillLightTables => fill_light_tables().apply(r),
                Call::KnownKind => r.eax = known_kind(r.eax),
                Call::LoadObjects => {
                    use crate::scene::load_front_end_objects_logged as logged;
                    *r = logged::load_front_end_objects_logged(r, &mut logged::game_part)?;
                }
                Call::ResetTexturePlaces => reset_texture_places().apply(r),
                Call::KnownKindPicture => r.eax = known_kind_picture(r.eax),
                Call::ReloadSystemTextures => call_game(r, game_addr(rva(0x4254B4))),
                Call::AllocatePointRecords => allocate_point_records_in_game()?.apply(r),
                Call::FillSceneLightTables => fill_scene_light_tables().apply(r),
                Call::SetViewMatrices => set_view_matrices().apply(r),
                Call::ReloadSlotTextures => reload_slot_textures_in_game(r)?,
            }
            Ok(())
        })
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    if let Err(e) = unsafe { set_up_front_end_scene_in_game(r) } {
        *r = entry;
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0xE07;
const ENTRY_BYTES: [u8; 11] = [0x60, 0xC7, 0x05, 0x34, 0x36, 0x62, 0x00, 0, 0, 0, 0];
/// The 50 calls; the one at 0x412 is to a function that only returns, left as it is.
const CALLS: [usize; 50] = [
    0x15, 0x1A, 0x16F, 0x174, 0x179, 0x21E, 0x22D, 0x3FB, 0x40D, 0x412, 0x417, 0x535, 0x55E, 0x588,
    0x5B2, 0x5DC, 0x61E, 0x660, 0x6A2, 0x6E4, 0x726, 0x768, 0x7AA, 0x7EC, 0x82E, 0x870, 0x8B2,
    0x8F4, 0x936, 0x978, 0x9BA, 0x9FC, 0xA3E, 0xA80, 0xAC2, 0xB04, 0xB46, 0xB88, 0xBCA, 0xC0C,
    0xC4E, 0xC90, 0xCD2, 0xD14, 0xD5B, 0xD60, 0xDA1, 0xDA6, 0xDAB, 0xDF6,
];
/// 49 calls in the listing; the one of `scene::known_kind` is made 0x22 times.
const CALLS_NOTED: usize = 82;

/// Everything the function itself writes.
fn regions() -> [Region; 13] {
    let table = |rva: usize, dwords: usize| (game_addr(rva) as u32, dwords * 4);
    [
        table(BAR_FROM_RVA, 2),
        at(BAR_FUNCTION_RVA),
        table(TAGS_RVA, TEXTURES),
        table(SYSTEM_LEVELS_RVA, TEXTURES),
        table(VIDEO_LEVELS_RVA, TEXTURES),
        table(DARK_COLOUR_RVA, 9),
        table(rva(0x609300), 5),
        at(rva(0x609320)),
        at(rva(0x609358)),
        at(OTHER_WAY_RVA),
        at(SET_UP_RVA),
        table(SLOT_KINDS_RVA, 0x100),
        table(KIND_PICTURES_RVA, TEXTURES),
    ]
}

/// Each call: its callee, the registers it was entered with, and a sum over everything
/// written by then.
static CALLED: Mutex<Vec<[u32; 9]>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    let sum = capture(&regions())
        .iter()
        .fold(0x811C_9DC5u32, |sum, &byte| {
            (sum ^ byte as u32).wrapping_mul(0x0100_0193)
        });
    CALLED
        .lock()
        .unwrap()
        .push([place, r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp, sum]);
    if place == Call::KnownKind as u32 || place == Call::KnownKindPicture as u32 {
        // These two answer in eax and keep the rest: the function counts and writes
        // through edi around them.
        r.eax = r.eax.wrapping_mul(0x31).wrapping_add(0x700 + place);
        return;
    }
    let turn =
        |value: u32, by: u32| value.rotate_left(by) ^ (0x9E37_79B9u32.wrapping_mul(place + 1));
    r.eax = turn(r.eax, 3);
    r.ebx = turn(r.ebx, 5);
    r.ecx = turn(r.ecx, 7);
    r.edx = turn(r.edx, 11);
    r.esi = turn(r.esi, 13);
    r.edi = turn(r.edi, 17);
    r.ebp = turn(r.ebp, 19);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x474E04)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                CALLEES
                    .iter()
                    .position(|&known| known == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let switch = [at(rva(0x5CDADC))];
    let saved = (capture(&regions), capture(&switch));
    for (case, (fill, twice, eax)) in [
        (0xAAu8, 0u32, 0u32),
        (0x55, 1, 0x1234_5678),
        (0x00, 0, 0xFFFF_FFFF),
        (0xFF, 0x100, 0x8000_0001),
    ]
    .into_iter()
    .enumerate()
    {
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![fill; saved.0.len()]);
            game_write(rva(0x5CDADC), twice);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let mut r = Regs {
                eax,
                ..Regs::SENTINEL
            };
            if rust {
                let done = set_up_front_end_scene(&mut r, &mut |call, r| {
                    act(call as u32, r);
                    Ok(())
                });
                if done.is_err() {
                    r = Regs::default();
                }
            } else {
                call_game(&mut r, copy.addr());
            }
            (
                [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
                CALLED.lock().unwrap().clone(),
                capture(&regions),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        let differs = theirs
            .1
            .iter()
            .zip(&mine.1)
            .position(|(theirs, mine)| theirs != mine);
        t.check(theirs == mine && theirs.1.len() == CALLS_NOTED, || {
            format!(
                "case {case}: registers original {:x?}, rust {:x?}; calls {} and {}, the first that differs {differs:?}: {:x?} / {:x?}; memory same {}",
                theirs.0,
                mine.0,
                theirs.1.len(),
                mine.1.len(),
                differs.map(|n| theirs.1[n]),
                differs.map(|n| mine.1[n]),
                theirs.2 == mine.2,
            )
        });
    }
    restore(&regions, &saved.0);
    restore(&switch, &saved.1);
    t.finish()
}
