//! `screens::gearing_text` (0x4699E0): string 0x21F + the current player's `gearing`, as
//! a text of the car choice.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_7554F0` = the current player's (`dword_75598C`) `gearing`; the string and a NUL
//! into `byte_A8BF54`. In two entry lists (0x7593FE, 0x75941A). Refuses a current player
//! past four, a text longer than its place (0x100). Self-test: the game's player with values
//! of the test's, the game's strings.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::player::{Player, player_at};
use crate::menu::take_shown_car::CURRENT_PLAYER_RVA;
use crate::screens::car_caption::put_string;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::strings::string_table::blob;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::gearing_text", rva(0x4699E0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const KEPT_RVA: usize = rva(0x7554F0); // dword_7554F0
pub const TEXT_RVA: usize = rva(0xA8BF54); // byte_A8BF54
pub const TEXT_LEN: usize = 0x100;
const FIRST_STRING: u32 = 0x21F;

const _: () = assert!(std::mem::offset_of!(Player, gearing) == 0x30);

pub fn current() -> Result<u32, Refused> {
    let n: u32 = unsafe { game_read(CURRENT_PLAYER_RVA) };
    let p = player_at(n);
    if n >= 4 || p == 0 {
        return Err(BadInput("a current player past the four").into());
    }
    Ok(p)
}

pub unsafe fn gearing_text() -> Result<(), Refused> {
    let value = unsafe { (*(current()? as *const Player)).gearing };
    unsafe {
        put_string(FIRST_STRING.wrapping_add(value), TEXT_RVA, TEXT_LEN)?;
        game_write(KEPT_RVA, value);
    }
    Ok(())
}

/// `menu::next_gearing` / `menu::previous_gearing`: `dword_7554F0` on (past 2 to 0) or back
/// (below 0 to 2, both signed), the current player's `gearing` it, and its text.
pub unsafe fn step_gearing(on: bool) -> Result<(), Refused> {
    let p = current()?;
    unsafe {
        let kept: i32 = game_read(KEPT_RVA);
        let value = if on {
            let v = kept.wrapping_add(1);
            if v > 2 { 0 } else { v }
        } else {
            let v = kept.wrapping_sub(1);
            if v < 0 { 2 } else { v }
        } as u32;
        put_string(FIRST_STRING.wrapping_add(value), TEXT_RVA, TEXT_LEN)?;
        game_write(KEPT_RVA, value);
        (*(p as *mut Player)).gearing = value;
    }
    Ok(())
}

pub fn regions() -> [Region; 2] {
    [at(KEPT_RVA), (game_addr(TEXT_RVA) as u32, TEXT_LEN)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { gearing_text() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if blob().is_null() {
        return t.skip("string table not loaded");
    }
    let Ok(p) = current() else {
        return t.skip("no current player");
    };
    let mut all = regions().to_vec();
    all.push((p, std::mem::size_of::<Player>()));
    let saved = capture(&all);
    for value in 0..4u32 {
        check_state(
            &mut t,
            &format!("gearing {value}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&all, &saved);
                std::ptr::write_bytes(game_addr(TEXT_RVA) as *mut u8, 0xAA, TEXT_LEN);
                (*(p as *mut Player)).gearing = value;
            },
            &regions(),
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
