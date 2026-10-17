//! `race::set_up_mode_0_cars` (0x4364BD): `sub_43603C`'s set-up for `dword_62389C` 0:
//! `race::reset_car_blocks`; with `dword_52A910` 0 the two-player rule, the players'
//! kinds, both players' automatic set-ups and their +0x3C from `dword_75AD2C` /
//! `dword_75AE98` (below 0: 0, 100 or more: 0x100, else x 2.56 rounded); else every
//! race car's automatic set-up, then the players' kinds.
//!
//! In:  nothing
//! Out: the registers as the steps leave them (esi the second player's block; with
//!      `dword_52A910`, ecx the count)
//!
//! See `race/race_cars.rs`. One caller, `sub_43603C`. It begins with its call: private
//! copy, its seven calls relocated, manual patch. Refuses, with `dword_52A910` set, no
//! cars or more than 8 (the original's `jnz` would run on), before anything. Shadow mode
//! runs the original alone (the set-ups log). Self-test: a copy whose calls note
//! themselves, on the game's blocks (filled, put back), both paths, 1 .. 8 cars, the
//! values round 0 and 100; the refusal on the Rust version alone.

use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::race::objects::CARS_RVA;
use crate::race::race_cars::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

const CALLS: &[usize] = &[0x00, 0x12, 0x17, 0x22, 0x2D, 0xB8, 0xC7];

pub static HOOK: RawHook =
    RawHook::new("race::set_up_mode_0_cars", rva(0x4364BD), thunk, &ORIGINAL)
        .private_copy(0xCD)
        .relocate_calls(CALLS)
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { set_up_mode_0_cars(r, &mut cars_part) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x4364BD,
        0xCD,
        &[0xE8, 0xC8, 0x00, 0x00, 0x00],
        CALLS,
        CAR_CALLEES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = set_up_regions();
    let saved = capture(&regions);
    for (all, cars, brakes) in [
        (0u32, 2u32, [0i32, 0i32]),
        (0, 1, [-5, 99]),
        (0, 2, [50, 100]),
        (0, 2, [1, i32::MAX]),
        (0, 2, [i32::MIN, 37]),
        (1, 1, [0, 0]),
        (1, 3, [0, 0]),
        (2, 8, [0, 0]),
    ] {
        run_cars(
            &mut t,
            &format!("all {all}, {cars} cars, values {brakes:?}"),
            &copy,
            &regions,
            Regs::SENTINEL,
            &|| {
                prepare(0x0F0F_0000 ^ cars, 0, 2, cars, [0, 1]);
                prepare_set_up(0x22, all, brakes);
            },
            &|r| unsafe { set_up_mode_0_cars(r, &mut test_part) },
        );
    }
    unsafe {
        prepare_set_up(0x22, 1, [0, 0]);
        game_write(CARS_RVA, 0u32);
    }
    let refused = unsafe { set_up_mode_0_cars(&Regs::SENTINEL, &mut test_part) }.is_err();
    t.check(refused, || "no cars, all set up: not refused".to_string());
    restore(&regions, &saved);
    t.finish()
}
