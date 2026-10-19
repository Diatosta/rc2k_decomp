//! `race::show_replay_still` (0x46062E): the viewer's still frame, the block read and the
//! frame shown (`dword_5C2534`) drawn: `dword_601024` and `dword_6416BC` 1 (a still); the
//! block (`dword_9613A4`, 0x2810 bytes) into the replay's frame (`unk_7172EC`), its state
//! loaded (`drive::load_replay_cars`) and its cameras (`drive::replay_cameras`); the cars
//! stepped to the frame (`race::step_replay_cars` per frame, `dword_6EF7E4` the frame,
//! `dword_6EF7E0` the block); at frame 0 one step with `dword_6EF7CC` 1 and the block's
//! state loaded again; then each car (`g_currentCar`) loaded (`drive::load_car`), its
//! camera worked out when it runs and is not 2 (`physics::camera_frame`, `dword_6EF7CC`
//! 1), stored (`drive::store_car`); `dword_63DB74`, `dword_63DB78` 0, `dword_6416B8` 1,
//! the race's scene drawn (`sub_454229`: `race::draw_race_scene`) with `dword_6235E8` 1
//! (put back after), the three switches 0.
//!
//! In:  nothing
//! Out: the registers as the scene's drawing leaves them; no flags read
//!
//! Caller: `race::viewer_frame`. Refuses (before anything) a cars' count of 0 or past 8.
//! Its calls through `part` (`CALLEES`); in the game their Rust versions (`sub_454229`,
//! six bytes calling `draw_race_scene`, is not hooked: its Rust version is called). Shadow
//! mode runs the original alone (it draws). Self-test: a copy whose calls go to stand-ins,
//! against the Rust version: frames 0, 1 and 5, cars running or not, car kinds 0 .. 2.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::race::step_replay_cars::{CARS, CURRENT};
use crate::race::viewer::{
    BLOCK, BLOCK_DATA, FRAME, Part, Saved, answer, dispatch_parts, dword, move_dwords, region,
    run_both, test_part,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::show_replay_still", rva(0x46062E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The calls by place, as IDA shows them.
pub const CALLEES: [usize; 8] = [
    0, 0x46010D, // 1 drive::load_replay_cars
    0x460949, // 2 drive::replay_cameras
    0x460835, // 3 race::step_replay_cars
    0x460C40, // 4 drive::load_car
    0x44FF1B, // 5 physics::camera_frame
    0x460C5D, // 6 drive::store_car
    0x454229, // 7 sub_454229 (race::draw_race_scene)
];
const CALLS: [usize; 9] = [0x27, 0x2C, 0x51, 0x74, 0x97, 0xA7, 0xC8, 0xD7, 0x11E];

const STILL: usize = 0x601024; // dword_601024
const STILL_2: usize = 0x6416BC; // dword_6416BC
const DRAWING: usize = 0x6416B8; // dword_6416B8
const REPLAY_FRAME: usize = 0x7172EC; // unk_7172EC
const STEP_FRAME: usize = 0x6EF7E4; // dword_6EF7E4
const STEP_BLOCK: usize = 0x6EF7E0; // dword_6EF7E0
const STEP_FIRST: usize = 0x6EF7CC; // dword_6EF7CC
const KIND: usize = 0x71A234; // unk_71A234 (0 parked, 2 not followed)
const SWITCHES: [usize; 2] = [0x63DB74, 0x63DB78];
const SCENE_SWITCH: usize = 0x6235E8; // dword_6235E8

/// The Rust version: the registers it leaves.
pub fn show_replay_still(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    if !(1..=8).contains(&g(CARS)) {
        return Err(BadInput("a cars' count of 0 or past 8").into());
    }
    let mut r = *entry;
    p(STILL, 1);
    p(STILL_2, 1);
    let load_frame = |r: &mut Regs| {
        move_dwords(REPLAY_FRAME, BLOCK_DATA, 0xA04);
        r.ecx = 0;
        r.esi = a(BLOCK_DATA) + 0x2810;
        r.edi = a(REPLAY_FRAME) + 0x2810;
    };
    load_frame(&mut r);
    part(1, &mut r)?;
    part(2, &mut r)?;
    r.ecx = 0;
    while r.ecx != g(FRAME) {
        p(STEP_FRAME, r.ecx);
        p(STEP_BLOCK, g(BLOCK));
        let mut s = r;
        part(3, &mut s)?;
        r.ecx += 1;
    }
    if g(FRAME) == 0 {
        p(STEP_FIRST, 1);
        let mut s = r;
        part(3, &mut s)?;
        p(STEP_FIRST, 0);
        load_frame(&mut r);
        part(1, &mut r)?;
    }
    p(CURRENT, 0);
    loop {
        let mut s = r;
        part(4, &mut s)?;
        if !matches!(g(KIND), 0 | 2) {
            p(STEP_FIRST, 1);
            part(5, &mut s)?;
            p(STEP_FIRST, 0);
        }
        part(6, &mut s)?;
        p(CURRENT, g(CURRENT).wrapping_add(1));
        r.eax = g(CURRENT);
        if r.eax == g(CARS) {
            break;
        }
    }
    for at in SWITCHES {
        p(at, 0);
    }
    p(DRAWING, 1);
    let kept = g(SCENE_SWITCH);
    p(SCENE_SWITCH, 1);
    part(7, &mut r)?;
    p(SCENE_SWITCH, kept);
    p(DRAWING, 0);
    p(STILL_2, 0);
    p(STILL, 0);
    Ok(r)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::drive::*;
    unsafe {
        match place {
            1 => load_replay_cars::load_replay_cars().apply(r),
            2 => replay_cameras::replay_cameras(r)?.apply(r),
            3 => {
                *r = crate::race::step_replay_cars::step_replay_cars(
                    r,
                    &mut crate::race::step_replay_cars::game_part,
                )?
            }
            4 => load_car::load_car()?.apply(r),
            5 => {
                *r = crate::physics::camera_frame::camera_frame(
                    r,
                    &mut crate::physics::reset_parts::game_part,
                )?
            }
            6 => store_car::store_car()?.apply(r),
            _ => {
                *r = crate::race::draw_race_scene::draw_race_scene(
                    r,
                    &mut crate::race::draw_race_scene::game_part,
                )?
            }
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        show_replay_still(r, &mut game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places: &'static [(usize, u32)] = Box::leak(
        (1..8u32)
            .map(|p| (CALLEES[p as usize], p))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    // mov dword_601024, 1
    let copy = match crate::race::viewer::copy(
        0x46062E,
        0x148,
        &[0xC7, 0x05, 0x24, 0x10, 0x60, 0x00, 0x01, 0x00, 0x00, 0x00],
        &CALLS,
        places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [
        dword(STILL),
        dword(STILL_2),
        dword(DRAWING),
        region(REPLAY_FRAME, 0x2810),
        dword(STEP_FRAME),
        dword(STEP_BLOCK),
        dword(STEP_FIRST),
        dword(CURRENT),
        dword(SWITCHES[0]),
        dword(SWITCHES[1]),
        dword(SCENE_SWITCH),
        dword(KIND),
        dword(CARS),
        dword(BLOCK),
        dword(FRAME),
    ];
    let mut all = regions.to_vec();
    all.push(region(BLOCK_DATA, 0x2810));
    let saved = Saved::new(&all);
    for (frame, cars) in [(0u32, 2u32), (1, 1), (5, 4)] {
        run_both(
            &mut t,
            &format!("frame {frame}, {cars} cars"),
            copy.addr(),
            Regs::SENTINEL,
            &regions,
            0,
            &|| {
                p(FRAME, frame);
                p(BLOCK, 3);
                p(CARS, cars);
                p(SCENE_SWITCH, 7);
                crate::race::viewer::fill(BLOCK_DATA, 0x2810, 23);
                answer(Box::new(|place, r| {
                    if place == 4 {
                        p(KIND, g(CURRENT) % 3);
                    }
                    r.eax = r.eax.wrapping_add(place);
                }));
            },
            &|r| show_replay_still(r, &mut test_part),
        );
    }
    crate::race::viewer::quiet();
    drop(saved);
    t.finish()
}
