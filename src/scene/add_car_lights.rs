//! `scene::add_car_lights` (0x44F1FE): which cars' lamps light an object.
//!
//! In:  nothing
//! Out: ecx = the number of cars, esi = the last car's block; everything else preserved
//!
//! Goes through the `dword_63DB90` blocks of `g_carBlocks` and calls
//! `scene::add_car_light` for:
//! - with `dword_52A910` set: the car `dword_52A920`, or every car while
//!   `dword_6238B8` is 1;
//! - else: the cars under `dword_63DB94` (every car while `dword_6238B8` is 1), but
//!   not the car `dword_639968` (the one being drawn, by its use), and of those all
//!   of them if `dword_63DB94` is 1 or `dword_62389C` is 0 or 2, else only the car
//!   `dword_63DBA8[dword_63DB9C]`.
//!
//! One caller, `scene::gather_lights`.
//! Refuses no cars (the original goes through 2^32) or more than the eight blocks.
//! Shadow mode runs the original alone (each car is compared in
//! `scene::add_car_light`'s hook).
//! Self-test: a redirected copy whose two calls note the car.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    ECX, ESI, RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva,
    shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::add_car_light::add_car_light;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::add_car_lights", rva(0x44F1FE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const CARS_RVA: usize = rva(0x63DB90); // dword_63DB90
const BLOCKS_RVA: usize = rva(0x71B3B8); // g_carBlocks
const BLOCKS: u32 = 8;
const ONE_CAR_RVA: usize = rva(0x52A910); // dword_52A910
const THE_CAR_RVA: usize = rva(0x52A920); // dword_52A920
const ALL_CARS_RVA: usize = rva(0x6238B8); // dword_6238B8
const NEAR_CARS_RVA: usize = rva(0x63DB94); // dword_63DB94
pub const DRAWN_CAR_RVA: usize = rva(0x639968); // dword_639968
const VIEW_RVA: usize = rva(0x62389C); // dword_62389C
const VIEW_CAR_RVA: usize = rva(0x63DB9C); // dword_63DB9C
const VIEW_CARS_RVA: usize = rva(0x63DBA8); // dword_63DBA8

// ---- Rust version ----

/// The game's variables the choice reads.
struct GameCars;

impl rc2k_core::scene::lamps::CarLights for GameCars {
    fn one_car(&self) -> bool {
        unsafe { game_read::<u32>(ONE_CAR_RVA) != 0 }
    }
    fn the_car(&self) -> u32 {
        unsafe { game_read(THE_CAR_RVA) }
    }
    fn all_cars(&self) -> bool {
        unsafe { game_read::<u32>(ALL_CARS_RVA) == 1 }
    }
    fn near(&self) -> u32 {
        unsafe { game_read(NEAR_CARS_RVA) }
    }
    fn view(&self) -> u32 {
        unsafe { game_read(VIEW_RVA) }
    }
    fn drawn(&self) -> u32 {
        unsafe { game_read(DRAWN_CAR_RVA) }
    }
    fn viewed(&self) -> u32 {
        unsafe {
            game_read(VIEW_CARS_RVA.wrapping_add((game_read::<u32>(VIEW_CAR_RVA) as usize) << 2))
        }
    }
}

// Converted: `rc2k_core::scene::lamps::car_lights`
pub unsafe fn add_car_lights(
    add: &mut dyn FnMut(usize) -> Result<(), Refused>,
) -> Result<Left, Refused> {
    unsafe {
        let cars: u32 = game_read(CARS_RVA);
        if cars == 0 || cars > BLOCKS {
            return Err(BadInput("no cars, or more than their blocks").into());
        }
        let mut block = 0;
        for car in 0..cars {
            block = game_read::<u32>(BLOCKS_RVA + car as usize * 4);
            if rc2k_core::scene::lamps::car_lights(&GameCars, car) {
                add(block as usize)?;
            }
        }
        Ok(Left::nothing().with(ECX, cars).with(ESI, block))
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { add_car_lights(&mut |car| Ok(add_car_light(car)?)) } {
        Ok(left) => left.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

const LEN: usize = 0xA1;
const ENTRY_BYTES: [u8; 9] = [0x83, 0x3D, 0x10, 0xA9, 0x52, 0x00, 0x00, 0x74, 0x30];
const CALLS: [usize; 2] = [0x29, 0x91];

/// The cars the lamps were asked for.
static CALLED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push(r.esi);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x44F1FE)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == rva(0x44F29F)).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let inputs = [
        at(CARS_RVA),
        (game_addr(BLOCKS_RVA) as u32, BLOCKS as usize * 4),
        at(ONE_CAR_RVA),
        at(THE_CAR_RVA),
        at(ALL_CARS_RVA),
        at(NEAR_CARS_RVA),
        at(DRAWN_CAR_RVA),
        at(VIEW_RVA),
        at(VIEW_CAR_RVA),
        (game_addr(VIEW_CARS_RVA) as u32, 2 * 4),
    ];
    let saved = capture(&inputs);
    let mut seed = 0x0C0F_FEE5u32;
    let mut lot = move |below: u32| {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 10) % below
    };
    for case in 0..300u32 {
        let set: [(usize, u32); 10] = [
            (CARS_RVA, 1 + lot(BLOCKS)),
            (ONE_CAR_RVA, lot(2)),
            (THE_CAR_RVA, lot(5)),
            (ALL_CARS_RVA, lot(3)),
            (NEAR_CARS_RVA, lot(5)),
            (DRAWN_CAR_RVA, [u32::MAX, 0, 1, 2][lot(4) as usize]),
            (VIEW_RVA, lot(4)),
            (VIEW_CAR_RVA, lot(2)),
            (VIEW_CARS_RVA, lot(4)),
            (VIEW_CARS_RVA + 4, lot(4)),
        ];
        let run = |rust: bool| unsafe {
            for (at, value) in set {
                game_write(at, value);
            }
            for n in 0..BLOCKS {
                game_write(BLOCKS_RVA + n as usize * 4, 0x0B10_C000 + n * 0x100);
            }
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let before = Regs::SENTINEL;
            let left = if rust {
                add_car_lights(&mut |car| {
                    act(
                        0,
                        &mut Regs {
                            esi: car as u32,
                            ..Regs::default()
                        },
                    );
                    Ok(())
                })
                .ok()
                .map(|left| {
                    let mut r = before;
                    left.apply(&mut r);
                    (r.ecx, r.esi)
                })
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                // eax is a leftover of one of the compares.
                ro.preserved_except(&before, crate::framework::EAX | ECX | ESI)
                    .then_some((ro.ecx, ro.esi))
            };
            (left, CALLED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0.is_some(), || {
            format!("case {case}: {set:x?}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&inputs, &saved);
    t.finish()
}
