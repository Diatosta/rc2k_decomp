//! `scene::draw_car_object` (0x49017B): the 3D part of a frame of the car
//! specification screen.
//!
//! In:  nothing
//! Out: the registers as its parts leave them (the scene lists' caller keeps them all);
//!      nothing at all on a page other than the first
//!
//! Only with `dword_755998` (the page shown) at 0: `scene::clear_depth_buffer`,
//! `scene::assign_kind_textures`, `sub_490238`, `scene::map_texture_places`,
//! `scene::load_slot_textures`, `sub_4901A3`. The three under it have one caller each
//! and are inlined:
//! - `sub_490238`: with the carousel menu (`g_carouselMenu`) at 0xB, the entry before
//!   `dword_755990` (the car shown) and the three after it, wrapping at
//!   `g_carouselCount`, to `dword_75AAA8` .., then `scene::update_carousel_pictures`;
//! - `sub_4901A3` -> `sub_4901B3` -> `sub_49040D`: `dword_6365C4` = 0; the turn about y
//!   from the clock (times 8, sixteen bits), the place's z `dword_75A66C`, the kind
//!   `dword_75A674[dword_755990]` to `dword_75A658`, its object to `g_objectNumber`,
//!   `scene::draw_menu_object` with eax = it (between `pusha` and `popa`).
//!
//! In the scene list at 0x758F76; IDA had not made it a function.
//! Shadow mode runs the original alone.
//! Refuses an entry past the 0x100 kinds, and what its parts refuse.
//! Self-test: a redirected copy whose six calls note themselves on both pages, the
//! entries `sub_490238` picks for 40 cars by lot, and a redirected copy of
//! `sub_49040D` whose drawing notes what was set, 60 settings by lot.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::carousel_pictures::update;
use crate::scene::clear_depth_buffer::clear_depth_buffer_in_game;
use crate::scene::draw_carousel_entry::{
    ANGLE_Y_RVA, CLOCK_RVA, ENTRIES, KIND_RVA, KINDS_RVA, OBJECT_KINDS_RVA, OBJECT_RVA, PLACE_RVA,
    find_object_of_kind,
};
use crate::scene::draw_menu_object::draw_menu_object_in_game;
use crate::scene::load_slot_texture::load_slot_texture_in_game;
use crate::scene::load_slot_textures::load_slot_textures;
use crate::scene::map_texture_places::map_texture_places;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::draw_car_object", rva(0x49017B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const PAGE_RVA: usize = rva(0x755998); // dword_755998
const CAR_RVA: usize = rva(0x755990); // dword_755990: the entry shown
const MENU_RVA: usize = rva(0x75AA74); // g_carouselMenu
const COUNT_RVA: usize = rva(0x75AA78); // g_carouselCount
const PICKED_RVA: usize = rva(0x75AAA8); // dword_75AAA8 .. dword_75AAB4
const MODE_RVA: usize = rva(0x6365C4); // dword_6365C4
const Z_RVA: usize = rva(0x75A66C); // dword_75A66C
const STEPS: [usize; 6] = [
    rva(0x44E967),
    rva(0x4256D8),
    rva(0x490238),
    rva(0x42570D),
    rva(0x425778),
    rva(0x4901A3),
];

// ---- Rust version ----

/// `sub_490238`'s four entries around the car shown.
pub fn picked(car: u32, count: u32) -> [u32; 4] {
    let mut entry = if car == 0 { count } else { car }.wrapping_sub(1);
    let mut entries = [entry; 4];
    for next in &mut entries[1..] {
        entry = entry.wrapping_add(1);
        if entry == count {
            entry = 0;
        }
        *next = entry;
    }
    entries
}

/// `sub_490238`.
pub unsafe fn pick_pictures(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(MENU_RVA) != 0xB {
            return Ok(());
        }
        let entries = picked(game_read(CAR_RVA), game_read(COUNT_RVA));
        game_write(PICKED_RVA, entries);
        update(entries)?.apply(r);
        Ok(())
    }
}

/// `sub_49040D`: the car shown's object set up and drawn by `draw`.
pub unsafe fn draw_shown_car(
    r: &Regs,
    draw: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let car: u32 = game_read(CAR_RVA);
        if car >= ENTRIES {
            return Err(BadInput("a car shown past the carousel's kinds").into());
        }
        let clock: u32 = game_read(CLOCK_RVA);
        game_write(ANGLE_Y_RVA, (clock << 3) & 0xFFFF);
        game_write(PLACE_RVA + 8, game_read::<u32>(Z_RVA));
        let kind: u32 = game_read(KINDS_RVA + car as usize * 4);
        game_write(KIND_RVA, kind);
        let object = find_object_of_kind(kind);
        game_write(OBJECT_RVA, object);
        let mut inner = Regs {
            eax: object,
            ebx: car << 2,
            ..*r
        };
        draw(&mut inner)
    }
}

/// `step`: what stands for the six calls, by their order. Nothing on a page other than
/// the first.
pub unsafe fn draw_car_object(
    r: &mut Regs,
    step: &mut dyn FnMut(usize, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    if unsafe { game_read::<u32>(PAGE_RVA) } != 0 {
        return Ok(());
    }
    for n in 0..STEPS.len() {
        step(n, r)?;
    }
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    let done = unsafe {
        draw_car_object(r, &mut |n, r| {
            match n {
                0 => clear_depth_buffer_in_game(r)?,
                1 => assign_kind_textures()?.apply(r),
                2 => pick_pictures(r)?,
                3 => map_texture_places().apply(r),
                4 => {
                    let entry = *r;
                    load_slot_textures(&mut |slot| load_slot_texture_in_game(slot, &entry))?
                        .apply(r)
                }
                _ => {
                    game_write(MODE_RVA, 0u32);
                    draw_shown_car(r, &mut |r| draw_menu_object_in_game(r))?;
                }
            }
            Ok(())
        })
    };
    if let Err(e) = done {
        *r = entry;
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0x28;
const ENTRY_BYTES: [u8; 7] = [0x83, 0x3D, 0x98, 0x59, 0x75, 0x00, 0x00];
const CALLS: [usize; 6] = [0x09, 0x0E, 0x13, 0x18, 0x1D, 0x22];
const PICK: usize = 0x490238;
const PICK_LEN: usize = 0x64;
const PICK_ENTRY_BYTES: [u8; 7] = [0x83, 0x3D, 0x74, 0xAA, 0x75, 0x00, 0x0B];
const PICK_CALLS: [usize; 1] = [0x490296 - PICK];
const DRAW: usize = 0x49040D;
const DRAW_LEN: usize = 0x42;
const DRAW_ENTRY_BYTES: [u8; 6] = [0x60, 0xA1, 0x30, 0x38, 0x61, 0x00];
const DRAW_CALLS: [usize; 2] = [0x49043E - DRAW, 0x490448 - DRAW];

/// Each call: its place and eax, ecx as it was entered with.
static CALLED: Mutex<Vec<[u32; 3]>> = Mutex::new(Vec::new());
/// Each update or drawing: what was written by then, and eax and ebx.
static NOTED: Mutex<Vec<(Vec<u8>, u32, u32)>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push([place, r.eax, r.ecx]);
    r.eax = r.eax.wrapping_mul(7).wrapping_add(place);
    r.ecx ^= 1 << place;
}

fn drawn() -> [Region; 4] {
    [
        at(ANGLE_Y_RVA),
        at(PLACE_RVA + 8),
        at(KIND_RVA),
        at(OBJECT_RVA),
    ]
}

fn noting(place: u32, r: &mut Regs) {
    let what = if place == 0 {
        capture(&[(game_addr(PICKED_RVA) as u32, 16)])
    } else {
        capture(&drawn())
    };
    NOTED.lock().unwrap().push((what, r.eax, r.ebx));
    r.eax = 0x100;
    r.edx ^= 0x77;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copies = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x49017B)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                STEPS
                    .iter()
                    .position(|&step| step == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
        .and_then(|outer| {
            RedirectedCopy::new(
                game_addr(rva(PICK)),
                PICK_LEN,
                &PICK_ENTRY_BYTES,
                &PICK_CALLS,
                &|callee| (callee == rva(0x49029C)).then(|| stand_in(0)),
            )
            .map(|pick| (outer, pick))
        })
        .and_then(|(outer, pick)| {
            RedirectedCopy::new(
                game_addr(rva(DRAW)),
                DRAW_LEN,
                &DRAW_ENTRY_BYTES,
                &DRAW_CALLS,
                &|callee| (callee == rva(0x4907E7)).then(|| stand_in(1)),
            )
            .map(|draw| (outer, pick, draw))
        })
    };
    let (outer, pick, draw) = match copies {
        Ok(copies) => copies,
        Err(why) => return t.skip(why),
    };
    let inputs: [Region; 7] = [
        at(PAGE_RVA),
        at(CAR_RVA),
        at(MENU_RVA),
        at(COUNT_RVA),
        (game_addr(PICKED_RVA) as u32, 16),
        at(Z_RVA),
        at(CLOCK_RVA),
    ];
    let regions = drawn();
    let saved = (capture(&inputs), capture(&regions));
    // The six calls in their order, on the first page and on another.
    for page in [0u32, 1, 3] {
        let run = |rust: bool| unsafe {
            game_write(PAGE_RVA, page);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let before = Regs::SENTINEL;
            let mut r = before;
            let kept = if rust {
                draw_car_object(&mut r, &mut |n, r| {
                    act(n as u32, r);
                    Ok(())
                })
                .is_ok()
            } else {
                call_game(&mut r, outer.addr());
                true
            };
            (kept, [r.eax, r.ecx], CALLED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0, || {
            format!("page {page}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    let mut seed = 0x4901_7B00u32;
    let mut lot = |below: u32| {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) % below
    };
    // The four entries picked.
    for case in 0..40 {
        let menu = [0xBu32, 0xB, 0xB, 0xC][lot(4) as usize];
        let count = 1 + lot(20);
        let car = [0, count - 1, lot(count), lot(count)][lot(4) as usize];
        let run = |rust: bool| unsafe {
            game_write(MENU_RVA, menu);
            game_write(COUNT_RVA, count);
            game_write(CAR_RVA, car);
            game_write(PICKED_RVA, [0xAAAA_AAAAu32; 4]);
            NOTED.lock().unwrap().clear();
            set_handler(noting);
            let picked_by = if rust {
                if menu == 0xB {
                    let entries = picked(car, count);
                    game_write(PICKED_RVA, entries);
                    act(0, &mut { Regs::SENTINEL });
                }
                capture(&[(game_addr(PICKED_RVA) as u32, 16)])
            } else {
                let mut r = Regs::SENTINEL;
                call_game(&mut r, pick.addr());
                capture(&[(game_addr(PICKED_RVA) as u32, 16)])
            };
            (picked_by, NOTED.lock().unwrap().len())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine, || {
            format!(
                "case {case}: menu {menu:#x}, count {count}, car {car}: original {theirs:x?}; rust {mine:x?}"
            )
        });
    }
    // The car's object.
    let loaded: Vec<u32> = (1..0x100usize)
        .map(|n| unsafe { game_read::<u32>(OBJECT_KINDS_RVA + n * 4) })
        .filter(|&kind| kind != 0)
        .take(4)
        .collect();
    const CAR: u32 = 0xF3;
    let kinds = [at(KINDS_RVA), at(KINDS_RVA + CAR as usize * 4)];
    let saved_kinds = capture(&kinds);
    for case in 0..60 {
        let car = [0u32, CAR][lot(2) as usize];
        let kind = match lot(6) {
            0 => 0x259,
            1 => 0xFFFF_FFFF,
            n => loaded.get(n as usize - 2).copied().unwrap_or(0x230 + n),
        };
        let (z, clock) = (lot(0x100_0000), lot(0x00FF_FFFF) << 4 | lot(16));
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.1.len()]);
            game_write(CAR_RVA, car);
            game_write(Z_RVA, z);
            game_write(CLOCK_RVA, clock);
            game_write(KINDS_RVA + car as usize * 4, kind);
            NOTED.lock().unwrap().clear();
            set_handler(noting);
            let before = Regs::SENTINEL;
            let mut r = before;
            let kept = if rust {
                draw_shown_car(&r, &mut |r| {
                    act(1, r);
                    Ok(())
                })
                .is_ok()
            } else {
                call_game(&mut r, draw.addr());
                r.preserved_except(&before, 0)
            };
            (kept, capture(&regions), NOTED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 && theirs.2.len() == 1, || {
            format!(
                "case {case}: car {car:#x}, kind {kind:#x}, z {z:#x}, clock {clock:#x}: original {theirs:x?}; rust {mine:x?}"
            )
        });
    }
    restore(&kinds, &saved_kinds);
    restore(&inputs, &saved.0);
    restore(&regions, &saved.1);
    t.finish()
}
