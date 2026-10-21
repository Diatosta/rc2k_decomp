//! More item actions the standalone could reach unported (2026-10-19, the survey of the
//! menu and draw-list tables):
//! - the current player's (`g_players`[`g_currentPlayer`]) settings, every register kept
//!   unless said: `menu::keep_player_keys` (0x46AEFC): the 0x2C bytes at `unk_7555B1` to
//!   +0x50; `menu::lower_player_40` (0x46B386, no `pusha`: edi the player, eax the value):
//!   +0x40 one less while above 0 (signed); `menu::lower_player_44` (0x46B3CB): +0x44 one less
//!   unless 0; `menu::raise_player_48` (0x46B528): +0x48 ten more unless 100;
//!   `menu::lower_player_48` (0x46B54C): +0x48 ten less unless 0; `menu::toggle_player_3c`
//!   (0x4704EC): +0x3C's bit 0 turned over;
//! - `menu::next_scores_page` (0x46D0FB) and `menu::previous_scores_page` (0x46D115): the
//!   best scores' page (`dword_755240`) one more (11 back to 0) or less (below 0 to 10);
//! - `menu::previous_replay_range` (0x46D423) and `menu::next_replay_range` (0x46D43B): the
//!   replays' range (`dword_755248`) one less (below 0 to 4) or more (past 4 to 0), then its
//!   caption (`screens::replay_range_caption`);
//! - `menu::player_N_car_specs` (0x46FCC1, 0x46FCDD, 0x46FCF9, 0x46FD15): `menu::show_car_specs`
//!   for player N (0 .. 3, `g_currentPlayer` set and put back);
//! - `menu::toggle_caption_choice` (0x47056B): `dword_75AB30`'s bit 0 turned over and string
//!   0x4FC + it with a NUL into the caption (`g_captionText`; eax as
//!   `strings::append_string_by_id` leaves it, ebx past the NUL);
//! - `menu::leave_with_carry` (0x471CCE): `dword_755561` and `dword_755565` 1, the carry set;
//!
//! 0x48558A, `pusha; popa; retn` in a draw list, does nothing and is too short for a hook's
//! jump: the standalone counts it among the empty functions it skips.
//!
//! Refuses (before anything) no current player (or one past the four). The car specs run a
//! screen: ungated thunks, their call through `part` (in the game the hook, `call_game`).
//! The others compared in shadow mode by state. Self-test: each against the original: the
//! settings on the game's current player (its fields put back), the pages from each end, the
//! caption both ways; the car specs on a copy whose call goes to a stand-in.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, EBX, EDI, Regs, SelfTest, call_game, game_addr, game_read, game_write, rva,
};
use crate::menu::parts::{Part, copy, dispatch_parted, run_both, test_part};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use rc2k_core::menu::player::Player;
use rc2k_core::menu::steps;

const CURRENT_PLAYER_RVA: usize = rva(0x75598C); // g_currentPlayer
const KEYS_RVA: usize = rva(0x7555B1); // unk_7555B1: 0x2C bytes
const SCORES_PAGE_RVA: usize = rva(0x755240); // dword_755240
const REPLAY_RANGE_RVA: usize = rva(0x755248); // dword_755248
const CHOICE_RVA: usize = rva(0x75AB30); // dword_75AB30
const CAPTION_RVA: usize = rva(0xA8BE54); // g_captionText
const CAPTION_LEN: usize = 0x100;
const LEAVE_A_RVA: usize = rva(0x755561); // dword_755561
const LEAVE_B_RVA: usize = rva(0x755565); // dword_755565
const SHOW_CAR_SPECS: usize = 0x46FD31; // menu::show_car_specs

fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}
fn p(a: usize, v: u32) {
    unsafe { game_write(a, v) }
}
fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The current player's record.
fn player() -> Result<u32, Refused> {
    match crate::menu::player::player_at(g(CURRENT_PLAYER_RVA)) {
        0 => Err(BadInput("no current player (or one past the four)").into()),
        p => Ok(p),
    }
}

// ---- the Rust versions ----

// Converted: `rc2k_core::menu::player` (the player's fields) and `rc2k_core::menu::steps`
// (`scores_page`, `replay_range`, `flip`).

/// The current player's record changed by `f` (read, changed, written back whole).
fn with_player<R>(f: impl FnOnce(&mut Player) -> R) -> Result<(u32, R), Refused> {
    let at = player()?;
    unsafe {
        let mut p = (at as *const Player).read_unaligned();
        let r = f(&mut p);
        (at as *mut Player).write_unaligned(p);
        Ok((at, r))
    }
}

pub fn keep_player_keys() -> Result<(), Refused> {
    let controls = unsafe { game_read(KEYS_RVA) };
    with_player(|p| p.controls_50 = controls)?;
    Ok(())
}

/// edi the player, eax the value left.
pub fn lower_player_40() -> Result<(u32, u32), Refused> {
    with_player(Player::lower_40)
}

pub fn lower_player_44() -> Result<(), Refused> {
    with_player(Player::lower_44)?;
    Ok(())
}

pub fn raise_player_48() -> Result<(), Refused> {
    with_player(|p| p.step_48(true))?;
    Ok(())
}

pub fn lower_player_48() -> Result<(), Refused> {
    with_player(|p| p.step_48(false))?;
    Ok(())
}

pub fn toggle_player_3c() -> Result<(), Refused> {
    with_player(Player::toggle_3c)?;
    Ok(())
}

pub fn next_scores_page() {
    p(
        SCORES_PAGE_RVA,
        steps::scores_page(g(SCORES_PAGE_RVA), true),
    );
}

pub fn previous_scores_page() {
    p(
        SCORES_PAGE_RVA,
        steps::scores_page(g(SCORES_PAGE_RVA), false),
    );
}

pub fn previous_replay_range() {
    p(
        REPLAY_RANGE_RVA,
        steps::replay_range(g(REPLAY_RANGE_RVA), false),
    );
    unsafe { crate::screens::best_scores::replay_range_caption() };
}

pub fn next_replay_range() {
    p(
        REPLAY_RANGE_RVA,
        steps::replay_range(g(REPLAY_RANGE_RVA), true),
    );
    unsafe { crate::screens::best_scores::replay_range_caption() };
}

/// eax as `append_string_by_id` leaves it, ebx past the NUL.
pub fn toggle_caption_choice() -> (u32, u32) {
    let choice = steps::flip(g(CHOICE_RVA));
    p(CHOICE_RVA, choice);
    let mut cursor = game_addr(CAPTION_RVA) as u32;
    let caption = rc2k_core::screens::page_values::stage_options_caption(choice);
    let eax = unsafe {
        crate::strings::append_string_by_id::append_string_by_id(caption, &mut |at, n| {
            std::ptr::copy(at, cursor as *mut u8, n);
            cursor += n as u32;
        })
    };
    unsafe { (cursor as *mut u8).write(0) };
    (eax, cursor + 1)
}

pub fn leave_with_carry() {
    p(LEAVE_A_RVA, 1);
    p(LEAVE_B_RVA, 1);
}

pub unsafe fn player_car_specs(n: u32, entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let kept = g(CURRENT_PLAYER_RVA);
    p(CURRENT_PLAYER_RVA, n);
    let mut r = *entry;
    let done = part(0, &mut r);
    p(CURRENT_PLAYER_RVA, kept);
    done?;
    Ok(r)
}

fn car_specs_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { call_game(r, game_addr(rva(SHOW_CAR_SPECS))) };
    Ok(())
}

// ---- by state: the Rust versions on the registers ----

/// Each state action's Rust version as a `Left`, by its number.
fn state_rust(which: u32, r: &Regs) -> Result<Left, Refused> {
    Ok(match which {
        0 => {
            keep_player_keys()?;
            Left::nothing()
        }
        1 => {
            let (edi, eax) = lower_player_40()?;
            Left {
                carry: Some(false),
                zero: Some(eax == 0),
                ..Left::nothing().with(EDI, edi).with(EAX, eax)
            }
        }
        2 => {
            lower_player_44()?;
            Left::nothing()
        }
        3 => {
            raise_player_48()?;
            Left::nothing()
        }
        4 => {
            lower_player_48()?;
            Left::nothing()
        }
        5 => {
            toggle_player_3c()?;
            Left::nothing()
        }
        6 => {
            next_scores_page();
            Left::nothing()
        }
        7 => {
            previous_scores_page();
            Left::nothing()
        }
        8 => {
            previous_replay_range();
            Left::nothing()
        }
        9 => {
            next_replay_range();
            Left::nothing()
        }
        10 => {
            let (eax, ebx) = toggle_caption_choice();
            Left::nothing().with(EAX, eax).with(EBX, ebx)
        }
        11 => {
            leave_with_carry();
            Left::carry(true)
        }
        _ => {
            let _ = r;
            Left::nothing()
        }
    })
}

/// What each may write.
fn written(which: u32) -> Vec<Region> {
    let pl = crate::menu::player::player_at(g(CURRENT_PLAYER_RVA));
    match which {
        0..=5 if pl != 0 => vec![(pl + 0x3C, 0x40)],
        6 | 7 => vec![at(SCORES_PAGE_RVA)],
        8 | 9 => vec![
            at(REPLAY_RANGE_RVA),
            (game_addr(CAPTION_RVA) as u32, CAPTION_LEN),
        ],
        10 => vec![at(CHOICE_RVA), (game_addr(CAPTION_RVA) as u32, CAPTION_LEN)],
        11 => vec![at(LEAVE_A_RVA), at(LEAVE_B_RVA)],
        _ => vec![],
    }
}

fn state_test(
    hook: &crate::framework::RawHook,
    call_original: crate::screens::harness::CallOriginal,
    which: u32,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    if which <= 5 && crate::menu::player::player_at(g(CURRENT_PLAYER_RVA)) == 0 {
        return t.skip("no current player");
    }
    if (8..=10).contains(&which) && crate::strings::string_table::blob().is_null() {
        return t.skip("string table not loaded");
    }
    let regions = written(which);
    let saved = capture(&regions);
    let pl = crate::menu::player::player_at(g(CURRENT_PLAYER_RVA));
    // the starting values of the field each looks at
    let starts: Vec<u32> = match which {
        1 => vec![0, 1, 5, 0xFFFF_FFFF],
        2 => vec![0, 1, 7],
        3 | 4 => vec![0, 10, 50, 100],
        5 => vec![0, 1, 2, 3],
        6 | 7 => vec![0, 1, 9, 10],
        8 | 9 => vec![0, 1, 3, 4],
        10 => vec![0, 1],
        _ => vec![0],
    };
    for start in starts {
        let set = || match which {
            1 => put(pl + 0x40, start),
            2 => put(pl + 0x44, start),
            3 | 4 => put(pl + 0x48, start),
            5 => put(pl + 0x3C, start),
            6 | 7 => p(SCORES_PAGE_RVA, start),
            8 | 9 => p(REPLAY_RANGE_RVA, start),
            10 => p(CHOICE_RVA, start),
            _ => {}
        };
        check_state(
            &mut t,
            &format!("from {start:#x}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &regions,
            &|r| state_rust(which, r),
        );
    }
    restore(&regions, &saved);
    t.finish()
}

macro_rules! state_action {
    ($module:ident, $name:literal, $address:expr, $which:expr) => {
        pub mod $module {
            use super::*;
            use crate::framework::{RawHook, hook_asm};
            use std::sync::atomic::AtomicUsize;
            static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
            pub static HOOK: RawHook =
                RawHook::new($name, rva($address), thunk, &ORIGINAL).self_test(self_test);
            hook_asm!(
                thunk,
                call_original,
                dispatch = dispatch,
                original = ORIGINAL
            );
            extern "C" fn dispatch(r: &mut Regs) {
                dispatch_state(&HOOK, r, call_original, &written($which), &|r| {
                    state_rust($which, r)
                });
            }
            fn self_test() -> (usize, usize) {
                state_test(&HOOK, call_original, $which)
            }
        }
    };
}

state_action!(keep_player_keys_hook, "menu::keep_player_keys", 0x46AEFC, 0);
state_action!(lower_player_40_hook, "menu::lower_player_40", 0x46B386, 1);
state_action!(lower_player_44_hook, "menu::lower_player_44", 0x46B3CB, 2);
state_action!(raise_player_48_hook, "menu::raise_player_48", 0x46B528, 3);
state_action!(lower_player_48_hook, "menu::lower_player_48", 0x46B54C, 4);
state_action!(toggle_player_3c_hook, "menu::toggle_player_3c", 0x4704EC, 5);
state_action!(next_scores_page_hook, "menu::next_scores_page", 0x46D0FB, 6);
state_action!(
    previous_scores_page_hook,
    "menu::previous_scores_page",
    0x46D115,
    7
);
state_action!(
    previous_replay_range_hook,
    "menu::previous_replay_range",
    0x46D423,
    8
);
state_action!(
    next_replay_range_hook,
    "menu::next_replay_range",
    0x46D43B,
    9
);
state_action!(
    toggle_caption_choice_hook,
    "menu::toggle_caption_choice",
    0x47056B,
    10
);
state_action!(
    leave_with_carry_hook,
    "menu::leave_with_carry",
    0x471CCE,
    11
);

macro_rules! car_specs {
    ($module:ident, $name:literal, $address:expr, $n:expr, $entry:expr) => {
        pub mod $module {
            use super::*;
            use crate::framework::{RawHook, hook_asm};
            use std::sync::atomic::AtomicUsize;
            static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
            pub static HOOK: RawHook =
                RawHook::new($name, rva($address), thunk, &ORIGINAL).self_test(self_test);
            hook_asm!(
                thunk,
                call_original,
                dispatch = dispatch,
                original = ORIGINAL,
                ungated
            );
            extern "C" fn dispatch(r: &mut Regs) {
                dispatch_parted(
                    &HOOK,
                    r,
                    call_original,
                    &|r, part| unsafe { player_car_specs($n, r, part) },
                    car_specs_part,
                );
            }
            fn self_test() -> (usize, usize) {
                let Some(mut t) = SelfTest::begin(&HOOK) else {
                    return (0, 0);
                };
                let copy = match copy($address, 0x1C, &$entry, &[0x10], &[SHOW_CAR_SPECS], &[]) {
                    Ok(c) => c,
                    Err(why) => return t.skip(why),
                };
                let vars = [at(CURRENT_PLAYER_RVA)];
                let saved = capture(&vars);
                for current in [0u32, 2] {
                    run_both(
                        &mut t,
                        &format!("current player {current}"),
                        &copy,
                        &vars,
                        &|| p(CURRENT_PLAYER_RVA, current),
                        Regs::SENTINEL,
                        false,
                        &|r| unsafe { player_car_specs($n, r, &mut test_part) },
                    );
                }
                restore(&vars, &saved);
                t.finish()
            }
        }
    };
}

// push g_currentPlayer (FF 35 8C 59 75 00)
car_specs!(
    player_0_car_specs,
    "menu::player_0_car_specs",
    0x46FCC1,
    0,
    [0xFF, 0x35, 0x8C, 0x59, 0x75, 0x00]
);
car_specs!(
    player_1_car_specs,
    "menu::player_1_car_specs",
    0x46FCDD,
    1,
    [0xFF, 0x35, 0x8C, 0x59, 0x75, 0x00]
);
car_specs!(
    player_2_car_specs,
    "menu::player_2_car_specs",
    0x46FCF9,
    2,
    [0xFF, 0x35, 0x8C, 0x59, 0x75, 0x00]
);
car_specs!(
    player_3_car_specs,
    "menu::player_3_car_specs",
    0x46FD15,
    3,
    [0xFF, 0x35, 0x8C, 0x59, 0x75, 0x00]
);

/// The sixteen, for the registry.
pub static HOOKS: [&dyn crate::framework::GameHook; 16] = [
    &keep_player_keys_hook::HOOK,
    &lower_player_40_hook::HOOK,
    &lower_player_44_hook::HOOK,
    &raise_player_48_hook::HOOK,
    &lower_player_48_hook::HOOK,
    &toggle_player_3c_hook::HOOK,
    &next_scores_page_hook::HOOK,
    &previous_scores_page_hook::HOOK,
    &previous_replay_range_hook::HOOK,
    &next_replay_range_hook::HOOK,
    &toggle_caption_choice_hook::HOOK,
    &leave_with_carry_hook::HOOK,
    &player_0_car_specs::HOOK,
    &player_1_car_specs::HOOK,
    &player_2_car_specs::HOOK,
    &player_3_car_specs::HOOK,
];
