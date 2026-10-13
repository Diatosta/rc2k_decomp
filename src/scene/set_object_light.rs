//! `scene::set_object_light` (0x45143A): the light that falls on the object as a whole,
//! to be added to each of its vertices.
//!
//! In:  nothing
//! Out: the original leaves eax, ebx, ecx and esi changed where it gathers the lamps;
//!      its caller goes on without them, and the Rust version keeps them
//!
//! `dword_635068` (what `scene::vertex_light` adds) = 0, then by `g_lightKind`:
//! - 5: plain white;
//! - 1: `g_lightStrength` = 0; the lamps gathered; `g_lightColour` plus
//!   `dword_635054`;
//! - 2: the same, with `g_lightStrength` = 0 only while the key `byte_4F6B74` is down;
//! - 3: the same without touching the strength, and with `dword_639968` =
//!   `dword_60627C` while the lamps are gathered (a car is not lit by its own), -1
//!   afterwards;
//! - anything else: nothing more.
//!
//! "The lamps gathered" (`scene::gather_lights`, 0x451425, inlined): the object's
//! place `dword_63501C` .. to `g_litPlace`, `g_lightColour` = 0,
//! `scene::add_car_lights`, `scene::add_point_lights`. "Plus" is byte by byte
//! without overflow, as in `scene::vertex_light`.
//! One caller, `sub_45045D`.
//! Shadow mode: the Rust version, its writes put back, the original, compared.
//! Self-test: a redirected copy whose three calls of the gathering note what they
//! find and leave a colour.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::add_car_light::add_car_light;
use crate::scene::add_car_lights::{DRAWN_CAR_RVA, add_car_lights};
use crate::scene::add_point_lights::{LIGHT_COLOUR_RVA, LIT_PLACE_RVA, MMX_RVA, add_point_lights};
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::vertex_light::plus;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::set_object_light", rva(0x45143A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const ADDED_RVA: usize = rva(0x635068); // dword_635068
const KIND_RVA: usize = rva(0x62DAF4); // g_lightKind
const STRENGTH_RVA: usize = rva(0x635090); // g_lightStrength
const KEY_RVA: usize = rva(0x4F6B74); // byte_4F6B74
const PLACE_RVA: usize = rva(0x63501C); // dword_63501C, dword_635020, dword_635024
const BASE_RVA: usize = rva(0x635054); // dword_635054
const THIS_CAR_RVA: usize = rva(0x60627C); // dword_60627C

// ---- Rust version ----

/// `gather`: what stands for `scene::add_car_lights` and `scene::add_point_lights`.
pub unsafe fn set_object_light(
    gather: &mut dyn FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        game_write(ADDED_RVA, 0u32);
        let kind: u32 = game_read(KIND_RVA);
        match kind {
            5 => {
                game_write(ADDED_RVA, 0x00FF_FFFFu32);
                return Ok(());
            }
            1 => game_write(STRENGTH_RVA, 0u32),
            2 => {
                if game_read::<u8>(KEY_RVA) != 0 {
                    game_write(STRENGTH_RVA, 0u32);
                }
            }
            3 => {}
            _ => return Ok(()),
        }
        game_write(LIT_PLACE_RVA, game_read::<[u32; 3]>(PLACE_RVA));
        if kind == 3 {
            game_write(DRAWN_CAR_RVA, game_read::<u32>(THIS_CAR_RVA));
        }
        game_write(LIGHT_COLOUR_RVA, 0u32);
        gather()?;
        if kind == 3 {
            game_write(DRAWN_CAR_RVA, u32::MAX);
        }
        let mmx = game_read::<u32>(MMX_RVA) != 0;
        game_write(
            ADDED_RVA,
            plus(game_read(LIGHT_COLOUR_RVA), game_read(BASE_RVA), mmx),
        );
        Ok(())
    }
}

/// With the lamps' Rust versions.
pub unsafe fn set_object_light_in_game() -> Result<(), Refused> {
    unsafe {
        set_object_light(&mut || {
            add_car_lights(&mut |car| Ok(add_car_light(car)?))?;
            add_point_lights()?;
            Ok(())
        })
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions: Vec<Region> = [
        ADDED_RVA,
        STRENGTH_RVA,
        DRAWN_CAR_RVA,
        LIGHT_COLOUR_RVA,
        CHECKED_RVA,
        rva(0x639958), // the lamps' strength
        rva(0x62DA7C), // and colour
    ]
    .map(at)
    .to_vec();
    regions.push((game_addr(LIT_PLACE_RVA) as u32, 12));
    // What a car is seen at.
    regions.push((game_addr(rva(0x6365B8)) as u32, 12));
    regions
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        if let Err(e) = unsafe { set_object_light_in_game() } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let regions = regions();
    let before = capture(&regions);
    let mine = unsafe { set_object_light_in_game() }.map(|()| capture(&regions));
    restore(&regions, &before);
    unsafe { call_original(r) };
    let Ok(mine) = mine else {
        return;
    };
    let theirs = capture(&regions);
    if mine != theirs {
        HOOK.mismatch(|| {
            format!(
                "kind {}: before {before:x?}; original {theirs:x?}; rust {mine:x?}",
                unsafe { game_read::<u32>(KIND_RVA) }
            )
        });
    }
}

// ---- self-test ----

const LEN: usize = 0x1CF;
const ENTRY_BYTES: [u8; 10] = [0xC7, 0x05, 0x68, 0x50, 0x63, 0x00, 0, 0, 0, 0];
const CALLS: [usize; 3] = [0x6F, 0xEB, 0x17D];

/// What each gathering found: everything written by then.
static CALLED: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());
static LEFT_COLOUR: Mutex<u32> = Mutex::new(0);

fn handler(_: u32, _: &mut Regs) {
    // `scene::gather_lights` begins by clearing the colour, which in the copy is the
    // stand-in's to do; then the lamps add theirs.
    unsafe { game_write(LIGHT_COLOUR_RVA, 0u32) };
    CALLED.lock().unwrap().push(capture(&regions()));
    unsafe { game_write(LIGHT_COLOUR_RVA, *LEFT_COLOUR.lock().unwrap()) };
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x45143A)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == rva(0x451425)).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let inputs = [
        at(KIND_RVA),
        (game_addr(KEY_RVA) as u32, 1),
        (game_addr(PLACE_RVA) as u32, 12),
        at(BASE_RVA),
        at(THIS_CAR_RVA),
        at(MMX_RVA),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let mut cases = 0u32;
    for kind in 0..=6u32 {
        for key in [0u8, 1, 0x80] {
            for (colour, base) in [
                (0u32, 0u32),
                (0x0010_2030, 0x00F0_F0F0),
                (0x80FF_0001, 0x9001_FFFF),
            ] {
                cases += 1;
                let run = |rust: bool| unsafe {
                    restore(&regions, &vec![0xAA; saved.0.len()]);
                    game_write(KIND_RVA, kind);
                    game_write(KEY_RVA, key);
                    game_write(PLACE_RVA, [0x1111u32 + cases, 0x2222, 0x3333]);
                    game_write(BASE_RVA, base);
                    game_write(THIS_CAR_RVA, 3u32);
                    game_write(MMX_RVA, cases % 2);
                    *LEFT_COLOUR.lock().unwrap() = colour;
                    CALLED.lock().unwrap().clear();
                    set_handler(handler);
                    let done = if rust {
                        set_object_light(&mut || {
                            // The stand-in is given the scene as the real one finds it:
                            // the colour is cleared by the function before the lamps.
                            act(0, &mut Regs::default());
                            Ok(())
                        })
                        .is_ok()
                    } else {
                        call_game(&mut Regs::default(), copy.addr());
                        true
                    };
                    (done, capture(&regions), CALLED.lock().unwrap().clone())
                };
                let (theirs, mine) = (run(false), run(true));
                t.check(theirs == mine, || {
                    format!(
                        "kind {kind}, key {key:#x}, colour {colour:#x}, base {base:#x}, case {cases}: original {theirs:x?}; rust {mine:x?}"
                    )
                });
            }
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
