//! `menu::step_carousel` (0x46989B): moves a menu's turning objects on by the time passed.
//!
//! In:  nothing
//! Out: every register preserved
//! The time since the last call (the clock `dword_613830` less `dword_75AA84`, which
//! becomes the clock) is kept, held to 100, in `dword_75AA88`. Then the turn
//! (`sub_49044F`, inlined): while `dword_75A660` is not 0 the angle `dword_75AA80` is
//! moved by ((angle / 8) + 0x4000000) x the time / 20 (`sub_4904C7`), down for a
//! negative direction, up (from the angle negated) for a positive one. When it passes
//! zero it is 0 and the direction 0; after a turn upwards the object in front,
//! `dword_75AA7C`, is the next one, the first again after the last (`dword_75AA78`).
//! `menu::move_highlight_left` and `_right` start a turn.
//! In 17 lists of the screens with turning objects (the main menu's among them).
//! Refuses a time kept above 0x71C71C6, with which the division would trap; it cannot
//! come, the time being held to 100.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::step_carousel", rva(0x46989B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const CLOCK_RVA: usize = rva(0x613830); // dword_613830
const LAST_RVA: usize = rva(0x75AA84); // dword_75AA84
const PASSED_RVA: usize = rva(0x75AA88); // dword_75AA88
const DIRECTION_RVA: usize = rva(0x75A660); // dword_75A660
const ANGLE_RVA: usize = rva(0x75AA80); // dword_75AA80
const FRONT_RVA: usize = rva(0x75AA7C); // dword_75AA7C
const COUNT_RVA: usize = rva(0x75AA78); // dword_75AA78
const LONGEST: u32 = 100;

// ---- Rust version ----

/// `sub_4904C7`: how far the angle moves.
fn step(from: u32, passed: u32) -> Result<u32, BadInput> {
    let moved = ((from >> 3) + 0x400_0000) as u64 * passed as u64 / 20;
    u32::try_from(moved).map_err(|_| BadInput("a time so long that the division traps"))
}

pub unsafe fn step_carousel() -> Result<(), BadInput> {
    unsafe {
        let clock: u32 = game_read(CLOCK_RVA);
        let passed = clock.wrapping_sub(game_read(LAST_RVA)).min(LONGEST);
        let direction: i32 = game_read(DIRECTION_RVA);
        let angle: u32 = game_read(ANGLE_RVA);
        // (the angle, the direction, whether the next object comes to the front)
        let turned = match direction {
            0 => None,
            ..0 => {
                let (left, passed_zero) = angle.overflowing_sub(step(angle, passed)?);
                Some(if passed_zero {
                    (0, 0, false)
                } else {
                    (left, direction, false)
                })
            }
            _ => {
                let (left, passed_zero) =
                    angle.overflowing_add(step(angle.wrapping_neg(), passed)?);
                Some(if passed_zero {
                    (0, 0, true)
                } else {
                    (left, direction, false)
                })
            }
        };
        // Nothing has been written up to here.
        game_write(LAST_RVA, clock);
        game_write(PASSED_RVA, passed);
        if let Some((angle, direction, next)) = turned {
            game_write(ANGLE_RVA, angle);
            game_write(DIRECTION_RVA, direction);
            if next {
                let front = game_read::<u32>(FRONT_RVA).wrapping_add(1);
                let wrapped = front == game_read::<u32>(COUNT_RVA);
                game_write(FRONT_RVA, if wrapped { 0 } else { front });
            }
        }
        Ok(())
    }
}

// ---- hook ----

fn regions() -> [Region; 5] {
    [
        at(LAST_RVA),
        at(PASSED_RVA),
        at(DIRECTION_RVA),
        at(ANGLE_RVA),
        at(FRONT_RVA),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { step_carousel() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: the game's variables set by the test, and put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let inputs = [at(CLOCK_RVA), at(COUNT_RVA)];
    let saved = (capture(&regions), capture(&inputs));
    let angles = [
        0u32,
        1,
        0x1000,
        0x0400_0000,
        0x4000_0000,
        0x8000_0000,
        0xC000_0000,
        0xFFFF_F000,
        0xFFFF_FFFF,
    ];
    // (the clock, the last clock, the objects, the one in front)
    let times = [
        (1000u32, 990u32, 5u32, 2u32),
        (1000, 0, 5, 4),
        (5, 0xFFFF_FFF0, 1, 0),
        (7, 7, 3, 2),
    ];
    for direction in [0i32, 1, -1, 0x7FFF_FFFF, i32::MIN] {
        for angle in angles {
            for (clock, last, count, front) in times {
                check_state(
                    &mut t,
                    &format!(
                        "direction {direction}, angle {angle:#x}, clock {clock} after {last}, object {front} of {count}"
                    ),
                    call_original,
                    Regs::SENTINEL,
                    &|| unsafe {
                        restore(&regions, &vec![0xAA; saved.0.len()]);
                        game_write(CLOCK_RVA, clock);
                        game_write(LAST_RVA, last);
                        game_write(COUNT_RVA, count);
                        game_write(FRONT_RVA, front);
                        game_write(DIRECTION_RVA, direction);
                        game_write(ANGLE_RVA, angle);
                    },
                    &regions,
                    &rust,
                );
            }
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
