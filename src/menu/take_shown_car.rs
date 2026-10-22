//! `menu::take_shown_car` (0x47B29D): the car shown on the car choice given to the current
//! player.
//!
//! In:  esi = the car list (pointers to the cars' entries)
//! Out: every register preserved
//!
//! The current player (`dword_75598C`) gets the entry `dword_755990` of the list: its +0 as
//! `car_number` (+8), its +8 as `no_turbo` (+0x90). Callers: `sub_482382` (after
//! `menu::select_car_loop`, esi the list it was given) and seven more. Refuses a current
//! player past the four (the original reads past `g_players`).
//! Self-test: a list and entries of the test's, the game's players, 8 cases.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::player::{PLAYERS, Player, player_at, player_regions};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::take_shown_car", rva(0x47B29D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const CURRENT_PLAYER_RVA: usize = rva(0x75598C); // dword_75598C
pub const SHOWN_CAR_RVA: usize = rva(0x755990); // dword_755990

const _: () = assert!(std::mem::offset_of!(Player, car_number) == 8);
const _: () = assert!(std::mem::offset_of!(Player, no_turbo) == 0x90);

pub unsafe fn take_shown_car(list: u32) -> Result<(), Refused> {
    let current: u32 = unsafe { game_read(CURRENT_PLAYER_RVA) };
    if current >= PLAYERS {
        return Err(BadInput("a current player past the four").into());
    }
    let shown: u32 = unsafe { game_read(SHOWN_CAR_RVA) };
    // Converted: `rc2k_core::menu::player::Player::take_car`.
    unsafe {
        let entry = ((list.wrapping_add(shown.wrapping_mul(4))) as *const u32).read_unaligned();
        let number = (entry as *const u32).read_unaligned();
        let no_turbo = ((entry + 8) as *const u32).read_unaligned();
        let at = player_at(current) as *mut Player;
        let mut p = at.read_unaligned();
        p.take_car(number, no_turbo);
        at.write_unaligned(p);
    }
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { take_shown_car(r.esi) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &player_regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let players = player_regions();
    if players.len() != PLAYERS as usize {
        return t.skip("a player missing from g_players");
    }
    let mut all = players.clone();
    all.extend([at(CURRENT_PLAYER_RVA), at(SHOWN_CAR_RVA)]);
    let saved = capture(&all);
    // Three entries of three dwords, and the list.
    let entries: Box<[[u32; 3]; 3]> = Box::new([[3, 0xAA, 1], [0x12, 0xBB, 0], [0x21, 0xCC, 7]]);
    let list: Box<[u32; 3]> = Box::new([0, 1, 2].map(|n| entries[n].as_ptr() as u32));
    for player in 0..PLAYERS {
        for shown in [0u32, 2] {
            check_state(
                &mut t,
                &format!("player {player}, car {shown}"),
                call_original,
                Regs {
                    esi: list.as_ptr() as u32,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    restore(&all, &saved);
                    game_write(CURRENT_PLAYER_RVA, player);
                    game_write(SHOWN_CAR_RVA, shown);
                },
                &players,
                &rust,
            );
        }
    }
    restore(&all, &saved);
    drop((entries, list));
    t.finish()
}
