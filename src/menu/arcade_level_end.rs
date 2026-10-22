//! `menu::arcade_level_end` (0x48AD73): an arcade level's end. Unless the level was won
//! (`dword_758768` 1): tune 7 (`sound::play_tune_7`), `menu::screen_488f4c_loop` and
//! `dword_7587F4` 1. Won: tune 6, `menu::arcade_finale`; the first time the level is won
//! (`byte_758800[level]` 0, made 1) its secret car: esi `off_75A12E`,
//! `menu::car_specs_47af4c_loop`, `add_shown_car`, `remove_shown_car`; then
//! `saves::save_game_data`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! The callees get eax the level (from the won level's test on) and esi as above. The
//! screens, the tunes and the save at the game's addresses (their hooks choose), the car
//! steps' Rust versions. Refuses (before anything) a level past 5. Ungated thunk (the
//! finale runs a race's drive); shadow mode runs the original alone. Self-test: a copy
//! whose calls go to stand-ins, against the Rust version: lost, won the first time, won
//! again.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::parts::{Part, copy, dispatch_parted, run_both, test_part};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::arcade_level_end", rva(0x48AD73), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const CALLEES: [usize; 8] = [
    0x40F7DA, // 0 sound::play_tune_7
    0x488F4C, // 1 menu::screen_488f4c_loop
    0x40F7C8, // 2 sound::play_tune_6
    0x488155, // 3 menu::arcade_finale
    0x47AF4C, // 4 menu::car_specs_47af4c_loop
    0x486BD2, // 5 menu::add_shown_car
    0x486BF5, // 6 menu::remove_shown_car
    0x411499, // 7 saves::save_game_data
];
const WON_RVA: usize = rva(0x758768); // dword_758768: the place
const LEVEL_RVA: usize = rva(0x7587EC); // dword_7587EC
const RETRY_RVA: usize = rva(0x7587F4); // dword_7587F4
const DONE_RVA: usize = rva(0x758800); // byte_758800: a level won before
const SECRET_CARS_RVA: usize = rva(0x75A12E); // off_75A12E

/// The Rust version: the registers it leaves.
pub unsafe fn arcade_level_end(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe {
        let mut r = *entry;
        // Converted: `rc2k_core::menu::arcade` (`won`, `won_level`, `first_win`).
        use rc2k_core::menu::arcade::{first_win, won, won_level};
        if !won(game_read(WON_RVA)) {
            part(0, &mut r)?;
            part(1, &mut r)?;
            game_write(RETRY_RVA, 1u32);
            return Ok(*entry);
        }
        let level = won_level(game_read(LEVEL_RVA))?;
        part(2, &mut r)?;
        part(3, &mut r)?;
        r.eax = level;
        if first_win(game_read(DONE_RVA + level as usize)) {
            game_write(DONE_RVA + level as usize, 1u8);
            r.esi = game_addr(SECRET_CARS_RVA) as u32;
            part(4, &mut r)?;
            part(5, &mut r)?;
            part(6, &mut r)?;
        }
        part(7, &mut r)?;
    }
    Ok(*entry)
}

/// In the game: the car steps' Rust versions, the rest at the game's addresses.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::menu::arcade_steps::{add_shown_car, remove_shown_car};
    match place {
        3 => {
            *r = unsafe {
                crate::menu::arcade_finale::arcade_finale(
                    r,
                    &mut crate::menu::arcade_finale::game_part,
                )
            }?
        }
        5 => add_shown_car()?,
        6 => remove_shown_car(&mut || unsafe {
            crate::saves::list_car_numbers::list_car_numbers().map_err(Refused::from)
        })?,
        _ => crate::calls::call(rva(CALLEES[place as usize]), r),
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parted(
        &HOOK,
        r,
        call_original,
        &|r, part| unsafe { arcade_level_end(r, part) },
        game_part,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let calls = [0xA, 0xF, 0x20, 0x25, 0x45, 0x4A, 0x4F, 0x54];
    let copy = match copy(
        0x48AD73,
        0x5B,
        &[0x60, 0x83, 0x3D, 0x68, 0x87, 0x75, 0x00, 0x01],
        &calls,
        &CALLEES,
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: [Region; 4] = [
        at(WON_RVA),
        at(LEVEL_RVA),
        at(RETRY_RVA),
        (game_addr(DONE_RVA) as u32, 8),
    ];
    let saved = capture(&regions);
    for (won, level, done) in [
        (0u32, 2u32, 0u8),
        (2, 1, 0),
        (1, 3, 0),
        (1, 3, 1),
        (1, 0, 0),
        (1, 5, 7),
    ] {
        run_both(
            &mut t,
            &format!("place {won}, level {level}, won before {done}"),
            &copy,
            &regions,
            &|| unsafe {
                game_write(WON_RVA, won);
                game_write(LEVEL_RVA, level);
                game_write(RETRY_RVA, 0u32);
                for n in 0..8 {
                    game_write(DONE_RVA + n, if n == level as usize { done } else { 0x5A });
                }
            },
            Regs::SENTINEL,
            false,
            &|r| unsafe { arcade_level_end(r, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
