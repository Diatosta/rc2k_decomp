//! What the four redefine keys pages share ("REDEFINIR TECLADO", `defkeys.png`): the
//! twelve keys a player drives with (`Player::keys`, +0x1C), a copy of them the page works
//! on (`g_redefinedKeys`, 0x755595, filled by the page's set-up), and the key being
//! redefined.
//!
//! An item's action (`menu::choose_key_0` .. `_11`, 0x4682D3 ..) sets `g_redefiningKey`
//! (0x7556AC) to 1 and `g_redefinedKeyPlace` (0x7556B0) to its key's place; then once a
//! frame `redefine_key` takes the pending key:
//!   none          nothing;
//!   1 (Escape)    the copy goes to the player unchanged and the redefining ends;
//!   any other     its low seven bits; a key the copy already has gets the redefined
//!                 place's old key (the two swap); otherwise a key of the reserved list
//!                 (`g_reservedKeys`, 0x755621, ended by a NUL) is not taken and the page
//!                 goes on waiting; then the key goes to its place, the place becomes 999
//!                 (0x3E7), the copy goes to the player and the redefining ends.
//! The place 8 has no item on the pages (it is not drawn either).

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, SelfTest, game_addr, game_read, game_write, rva};
use crate::input::take_key::{PENDING_KEY_RVA, pending_key, take_key};
use crate::menu::player::{PLAYERS, Player, player_at};
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{Region, at, capture, check_state_with, restore};
use crate::service::state::PLAYER_INDEX_RVA;
use std::sync::atomic::Ordering;

pub use rc2k_core::menu::player::{KEY_ESCAPE, KEYS, Redefined};
pub const KEYS_RVA: usize = rva(0x755595); // g_redefinedKeys
pub const REDEFINING_RVA: usize = rva(0x7556AC); // g_redefiningKey
pub const PLACE_RVA: usize = rva(0x7556B0); // g_redefinedKeyPlace
pub const RESERVED_RVA: usize = rva(0x755621); // g_reservedKeys
pub const NO_PLACE: u32 = 999;
/// The reserved list's length is not known; the game's has 18 keys.
const RESERVED_ROOM: usize = 0x100;

const KEYS_OFFSET: u32 = std::mem::offset_of!(Player, keys) as u32;

/// The reserved keys, up to their NUL; `None` if there is none in `RESERVED_ROOM`.
fn reserved() -> Option<Vec<u8>> {
    let list =
        unsafe { std::slice::from_raw_parts(game_addr(RESERVED_RVA) as *const u8, RESERVED_ROOM) };
    list.iter()
        .position(|&b| b == 0)
        .map(|end| list[..end].to_vec())
}

pub use rc2k_core::menu::player::place_key;

/// One frame of redefining a key of `player` (0: refused).
pub unsafe fn redefine_key(player: u32) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(REDEFINING_RVA) == 0 {
            return Ok(());
        }
        // Everything that could be refused, before the key is taken.
        if player == 0 {
            return Err(BadInput("no player").into());
        }
        let place: u32 = game_read(PLACE_RVA);
        if place as usize >= KEYS {
            return Err(BadInput("a key's place of 12 or more").into());
        }
        let Some(reserved) = reserved() else {
            return Err(BadInput("a reserved list without its end").into());
        };
        // Converted: `rc2k_core::menu::player::redefine`.
        let work = game_addr(KEYS_RVA) as *mut [u8; KEYS];
        let Redefined::Done { keys, placed } =
            rc2k_core::menu::player::redefine(work.read(), place as usize, take_key(), &reserved)
        else {
            return Ok(());
        };
        work.write(keys);
        if placed {
            game_write(PLACE_RVA, NO_PLACE);
        }
        ((player + KEYS_OFFSET) as *mut [u8; KEYS]).write_unaligned(keys);
        game_write(REDEFINING_RVA, 0u32);
    }
    Ok(())
}

/// What a frame may write, but the pending key.
pub fn regions(player: u32) -> Vec<Region> {
    let mut regions = vec![
        (game_addr(KEYS_RVA) as u32, KEYS),
        at(REDEFINING_RVA),
        at(PLACE_RVA),
    ];
    if player != 0 {
        regions.push((player + KEYS_OFFSET, KEYS));
    }
    regions
}

pub fn rust(player: fn() -> u32) -> impl Fn(&Regs) -> Result<Left, Refused> {
    move |_| {
        unsafe { redefine_key(player()) }?;
        Ok(Left::nothing())
    }
}

/// The self-test of a key handler: for each player it can be given (`players`, the index
/// the page leaves in `dword_75598C`), keys of the kinds there are, at three places.
pub fn check_redefine(
    t: &mut SelfTest,
    call_original: CallOriginal,
    players: &[u32],
    player: fn() -> u32,
) {
    let Some(reserved) = reserved() else {
        t.check(false, || "the reserved list has no end".into());
        return;
    };
    let reserved_key = reserved.iter().copied().find(|&k| k > 1).unwrap_or(0x3B);
    let inputs = [at(PLAYER_INDEX_RVA), (game_addr(PENDING_KEY_RVA) as u32, 1)];
    let saved_inputs = capture(&inputs);
    let all: Vec<Region> = (0..PLAYERS)
        .map(player_at)
        .filter(|&p| p != 0)
        .flat_map(regions)
        .collect();
    let saved = capture(&all);
    let start: [u8; KEYS] = [
        0x48, 0x50, 0x4B, 0x4D, 0x1E, 0x2C, 0x39, 0x1D, 0, 0x2E, 0x19, 0x26,
    ];
    for &index in players {
        if player_at(index) == 0 {
            continue;
        }
        unsafe { game_write(PLAYER_INDEX_RVA, index) };
        let p = player();
        if p == 0 {
            continue;
        }
        let regions = regions(p);
        for redefining in [0u32, 1] {
            for place in [0u32, 5, 11] {
                for key in [0u8, KEY_ESCAPE, 0x10, 0x9E, 0x2C, 0x48, reserved_key, 0xFF] {
                    let prepare = || unsafe {
                        (game_addr(KEYS_RVA) as *mut [u8; KEYS]).write(start);
                        ((p + KEYS_OFFSET) as *mut [u8; KEYS]).write_unaligned([0x77; KEYS]);
                        game_write(REDEFINING_RVA, redefining);
                        game_write(PLACE_RVA, place);
                        pending_key().store(key, Ordering::SeqCst);
                    };
                    let rust = rust(player);
                    check_state_with(
                        t,
                        &format!(
                            "player {index}, redefining {redefining}, place {place}, key {key:#x}"
                        ),
                        call_original,
                        Regs::SENTINEL,
                        &prepare,
                        &regions,
                        &rust,
                        &|| vec![pending_key().load(Ordering::SeqCst)],
                    );
                }
            }
        }
    }
    restore(&all, &saved);
    restore(&inputs, &saved_inputs);
}
