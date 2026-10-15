//! `menu::find_player_car` (0x4708FD): the carousel turned to the current player's car.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/car_choice.rs`. Callers: `menu::select_car_loop` and
//! `menu::car_specs_47af4c_loop` at their set-up. Self-test: entries of the test's (with
//! their end mark) and the game's player, the car each of them and one that is not there.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::car_choice::{find_player_car, regions};
use crate::menu::player::{Player, player_at};
use crate::menu::take_shown_car::CURRENT_PLAYER_RVA;
use crate::scene::carousel_pictures::ENTRIES_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::find_player_car", rva(0x4708FD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { find_player_car() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let current: u32 = unsafe { game_read(CURRENT_PLAYER_RVA) };
    let p = player_at(current);
    if current >= 4 || p == 0 {
        return t.skip("no current player");
    }
    let cars = [3u32, 0, 0x16, 0x21, 5];
    let records: Box<[[u32; 4]; 5]> = Box::new(cars.map(|c| [c, 0x249, 0, 0]));
    let mut list: Vec<u32> = records.iter().map(|r| r.as_ptr() as u32).collect();
    list.push(u32::MAX);
    let player = (p, std::mem::size_of::<Player>());
    let mut all = regions().to_vec();
    all.extend([at(ENTRIES_RVA), player]);
    let saved = capture(&all);
    for car in cars.iter().copied().chain([0x99]) {
        check_state(
            &mut t,
            &format!("car {car:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&all, &saved);
                game_write(ENTRIES_RVA, list.as_ptr() as u32);
                (*(p as *mut Player)).car_number = car;
            },
            &regions(),
            &rust,
        );
    }
    restore(&all, &saved);
    drop((records, list));
    t.finish()
}
