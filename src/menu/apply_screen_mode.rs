//! `menu::apply_screen_mode` (0x4703C2): the graphics page's "apply": the screen mode chosen
//! set (`sub_4045D6` with eax 1, the platform's: DirectDraw modes). Taken: the fonts freed
//! (`gfx::free_fonts`, `gfx::free_glyph_tables`) and built again (`gfx::build_fonts`,
//! `gfx::build_menu_sprites`), the 3D tidied (`render3d_tidy`, `objects_tidy`,
//! `nullsub_6`, `release_textures`), `scene::set_up_front_end_scene`, tune 4, the
//! highlighted item (`dword_755525`) and `dword_74771C` 1, the menu's sound 2. Refused (a
//! set carry): `sub_4045D6` with eax 0 (the mode before, a reading), the 3D tidied, the
//! front end's scene, tune 4, the item and `dword_74771C` 3, the menu's sound 8.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! An item action of the graphics page (0x7548D4). Its calls through `part`: the screen mode
//! at the game's address (section 15), the glyph tables, the scene and the sound in Rust,
//! the rest at the game's addresses (their hooks choose). Shadow mode runs the original
//! alone. Self-test: a copy whose calls go to stand-ins, against the Rust version, the mode
//! taken and refused.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva};
use crate::menu::parts::{Answer, Part, answers, copy, dispatch_parted, run_both, test_part};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::apply_screen_mode", rva(0x4703C2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CALLEES: [usize; 12] = [
    0x4045D6, // 0 the screen mode (sub_4045D6)
    0x415C55, // 1 gfx::free_fonts
    0x474D12, // 2 gfx::free_glyph_tables
    0x415563, // 3 gfx::build_fonts
    0x47C4A1, // 4 gfx::build_menu_sprites
    0x4433A7, // 5 race::render3d_tidy
    0x44346B, // 6 race::objects_tidy
    0x44347E, // 7 nullsub_6
    0x443396, // 8 race::release_textures
    0x474E04, // 9 scene::set_up_front_end_scene
    0x40F792, // 10 sound::play_tune_4
    0x40F059, // 11 sound::play_menu_sound
];
const ITEM_RVA: usize = rva(0x755525); // dword_755525
const AFTER_RVA: usize = rva(0x74771C); // dword_74771C

/// The Rust version: the registers it leaves.
pub unsafe fn apply_screen_mode(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = 1;
    part(0, &mut r)?;
    let (taken, sound) = if r.carry() {
        r.eax = 0;
        part(0, &mut r)?;
        (3u32, 8u32)
    } else {
        for place in [1, 2, 3, 4] {
            part(place, &mut r)?;
        }
        (1, 2)
    };
    for place in [5, 6, 7, 8, 9, 10] {
        part(place, &mut r)?;
    }
    unsafe {
        game_write(ITEM_RVA, taken);
        game_write(AFTER_RVA, taken);
    }
    r.eax = sound;
    part(11, &mut r)?;
    Ok(*entry)
}

pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            2 => {
                *r = crate::gfx::free_glyph_tables::free_glyph_tables(
                    r,
                    &mut crate::gfx::free_glyph_tables::game_part,
                )?
            }
            9 => crate::scene::set_up_front_end_scene::set_up_front_end_scene_in_game(r)?,
            11 => crate::sound::play_menu_sound::play_menu_sound_in_game(r),
            _ => call_game(r, game_addr(rva(CALLEES[place as usize]))),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parted(
        &HOOK,
        r,
        call_original,
        &|r, part| unsafe { apply_screen_mode(r, part) },
        game_part,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let calls = [
        0x6, 0xD, 0x12, 0x17, 0x1C, 0x21, 0x26, 0x2B, 0x30, 0x35, 0x3A, 0x58, 0x61, 0x66, 0x6B,
        0x70, 0x75, 0x7A, 0x7F, 0x9D,
    ];
    let copy = match copy(
        0x4703C2,
        0xA4,
        &[0x60, 0xB8, 0x01, 0x00, 0x00, 0x00],
        &calls,
        &CALLEES,
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: [Region; 2] = [at(ITEM_RVA), at(AFTER_RVA)];
    let saved = capture(&regions);
    for refused in [false, true] {
        answers(&[(0, &[Answer::carry(refused), Answer::carry(false)])]);
        run_both(
            &mut t,
            &format!("mode refused {refused}"),
            &copy,
            &regions,
            &|| unsafe {
                game_write(ITEM_RVA, 0xAAAA_AAAAu32);
                game_write(AFTER_RVA, 0xBBBB_BBBBu32);
            },
            Regs::SENTINEL,
            false,
            &|r| unsafe { apply_screen_mode(r, &mut test_part) },
        );
    }
    answers(&[]);
    restore(&regions, &saved);
    t.finish()
}
