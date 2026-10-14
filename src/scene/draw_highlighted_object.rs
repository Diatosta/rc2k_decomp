//! `scene::draw_highlighted_object` (0x49004E): the 3D part of a frame of the menus
//! that show one object, the highlighted item's.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `scene::clear_depth_buffer`, `scene::assign_kind_textures`,
//! `scene::assign_menu_textures`, `scene::map_texture_places`,
//! `scene::load_slot_textures`, then `sub_49006F` (one caller): `dword_6365C4` = 0 and
//! `sub_49007F` (one caller), between `pusha` and `popa`:
//! - the turn about y from the clock (`g_sceneClock` times 8, its low sixteen bits)
//!   to `dword_635040`; the place's x (`dword_63501C`) 0 and its z (`dword_635024`)
//!   `dword_75A66C`;
//! - the kind the highlighted item (`dword_755525`, from 1) has in the carousel's
//!   kinds (`dword_75A674`); a kind above 0x258 takes its size (`dword_635038`) from
//!   the twelve-byte records at `unk_75950E`, and the y and the turn about x
//!   (`dword_635020`, `dword_63503C`) unless they are -1 there;
//! - the kind to `dword_75A658`, its first object (`scene::find_object_of_kind`) to
//!   `g_objectNumber`, and `scene::draw_menu_object` with eax = it.
//!
//! In twelve of the menus' scene lists (the options and the screens under them, by the
//! standalone program's run of 2026-10-14).
//! Shadow mode runs the original alone.
//! Refuses a highlighted item of 0 or past the 0x100 kinds, a kind past the records,
//! and what its parts refuse.
//! Self-test: a redirected copy whose six calls note themselves, and a redirected copy
//! of `sub_49007F` whose drawing notes what was set by then, 120 settings by lot.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::assign_menu_textures::assign_menu_textures;
use crate::scene::clear_depth_buffer::clear_depth_buffer_in_game;
use crate::scene::draw_carousel_entry::{
    ANGLE_X_RVA, ANGLE_Y_RVA, CLOCK_RVA, ENTRIES, KIND_RVA, KINDS_RVA, OBJECT_KINDS_RVA,
    OBJECT_RVA, OWN, OWN_RVA, PLACE_RVA, SIZE_RVA, find_object_of_kind,
};
use crate::scene::draw_menu_object::draw_menu_object_in_game;
use crate::scene::load_slot_texture::load_slot_texture_in_game;
use crate::scene::load_slot_textures::load_slot_textures;
use crate::scene::map_texture_places::map_texture_places;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::draw_highlighted_object",
    rva(0x49004E),
    thunk,
    &ORIGINAL,
)
.private_copy(LEN)
.relocate_calls(&CALLS)
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const MODE_RVA: usize = rva(0x6365C4); // dword_6365C4
const HIGHLIGHTED_RVA: usize = rva(0x755525); // dword_755525
const Z_RVA: usize = rva(0x75A66C); // dword_75A66C
const STEPS: [usize; 6] = [
    rva(0x44E967),
    rva(0x4256D8),
    rva(0x490DF3),
    rva(0x42570D),
    rva(0x425778),
    rva(0x49006F),
];

// ---- Rust version ----

/// `sub_49007F`: the highlighted item's object set up and drawn by `draw`.
pub unsafe fn draw_highlighted(
    r: &Regs,
    draw: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let item = game_read::<u32>(HIGHLIGHTED_RVA).wrapping_sub(1);
        if item >= ENTRIES {
            return Err(BadInput("a highlighted item of 0 or past the carousel's kinds").into());
        }
        let kind: u32 = game_read(KINDS_RVA + item as usize * 4);
        let own = (kind as i32 > 0x258).then(|| kind.wrapping_sub(0x259));
        if own.is_some_and(|own| own >= OWN) {
            return Err(BadInput("a kind past the records of the kinds' own places").into());
        }
        let clock: u32 = game_read(CLOCK_RVA);
        game_write(ANGLE_Y_RVA, (clock << 3) & 0xFFFF);
        game_write(PLACE_RVA + 8, game_read::<u32>(Z_RVA));
        game_write(PLACE_RVA, 0u32);
        if let Some(own) = own {
            let [size, y, angle]: [u32; 3] = game_read(OWN_RVA + own as usize * 12);
            game_write(SIZE_RVA, size);
            if y != u32::MAX {
                game_write(PLACE_RVA + 4, y);
            }
            if angle != u32::MAX {
                game_write(ANGLE_X_RVA, angle);
            }
        }
        game_write(KIND_RVA, kind);
        let object = find_object_of_kind(kind);
        game_write(OBJECT_RVA, object);
        let mut inner = Regs {
            eax: object,
            ebx: item,
            ..*r
        };
        draw(&mut inner)
    }
}

/// `step`: what stands for the six calls, by their order.
pub unsafe fn draw_highlighted_object(
    r: &Regs,
    step: &mut dyn FnMut(usize, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let mut inner = *r;
    for n in 0..STEPS.len() {
        step(n, &mut inner)?;
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
    let done = unsafe {
        draw_highlighted_object(r, &mut |n, r| {
            match n {
                0 => clear_depth_buffer_in_game(r)?,
                1 => assign_kind_textures()?.apply(r),
                2 => assign_menu_textures()?,
                3 => map_texture_places().apply(r),
                4 => {
                    let entry = *r;
                    load_slot_textures(&mut |slot| load_slot_texture_in_game(slot, &entry))?
                        .apply(r)
                }
                _ => {
                    game_write(MODE_RVA, 0u32);
                    draw_highlighted(r, &mut |r| draw_menu_object_in_game(r))?;
                }
            }
            Ok(())
        })
    };
    if let Err(e) = done {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0x21;
const ENTRY_BYTES: [u8; 6] = [0x60, 0xE8, 0x13, 0xE9, 0xFB, 0xFF];
const CALLS: [usize; 6] = [0x01, 0x06, 0x0B, 0x10, 0x15, 0x1A];
const INNER: usize = 0x49007F;
const INNER_LEN: usize = 0x87;
const INNER_ENTRY_BYTES: [u8; 6] = [0x60, 0xA1, 0x30, 0x38, 0x61, 0x00];
const INNER_CALLS: [usize; 2] = [0x4900F5 - INNER, 0x4900FF - INNER];

/// Each call: its place and eax, ecx as it was entered with.
static CALLED: Mutex<Vec<[u32; 3]>> = Mutex::new(Vec::new());
/// Each drawing: everything set by then, and eax and ebx as it was entered with.
static DRAWN: Mutex<Vec<(Vec<u8>, u32, u32)>> = Mutex::new(Vec::new());

fn written() -> [Region; 6] {
    [
        (game_addr(PLACE_RVA) as u32, 12),
        at(SIZE_RVA),
        at(ANGLE_X_RVA),
        at(ANGLE_Y_RVA),
        at(KIND_RVA),
        at(OBJECT_RVA),
    ]
}

fn handler(place: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push([place, r.eax, r.ecx]);
    r.eax = r.eax.wrapping_mul(7).wrapping_add(place);
    r.ecx ^= 1 << place;
}

fn drawing(_: u32, r: &mut Regs) {
    DRAWN
        .lock()
        .unwrap()
        .push((capture(&written()), r.eax, r.ebx));
    r.eax = 0x100;
    r.edx ^= 0x77;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copies = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x49004E)),
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
                game_addr(rva(INNER)),
                INNER_LEN,
                &INNER_ENTRY_BYTES,
                &INNER_CALLS,
                &|callee| (callee == rva(0x4907E7)).then(|| stand_in(0)),
            )
            .map(|inner| (outer, inner))
        })
    };
    let (outer, inner) = match copies {
        Ok(copies) => copies,
        Err(why) => return t.skip(why),
    };
    // The six calls in their order.
    for ecx in [0u32, 0xFFFF_FFFF, 0x1234_5678] {
        let run = |rust: bool| unsafe {
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let before = Regs {
                ecx,
                ..Regs::SENTINEL
            };
            let kept = if rust {
                draw_highlighted_object(&before, &mut |n, r| {
                    act(n as u32, r);
                    Ok(())
                })
                .is_ok()
            } else {
                let mut r = before;
                call_game(&mut r, outer.addr());
                r.preserved_except(&before, 0)
            };
            (kept, CALLED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 && theirs.1.len() == 6, || {
            format!("ecx {ecx:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    // The highlighted item's object.
    let regions = written();
    const ITEM: u32 = 0xF7;
    let inputs: [Region; 6] = [
        at(HIGHLIGHTED_RVA),
        at(Z_RVA),
        at(CLOCK_RVA),
        at(KINDS_RVA + (ITEM as usize - 1) * 4),
        at(KINDS_RVA),
        (game_addr(OWN_RVA) as u32, 12),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let mut seed = 0x4900_7F00u32;
    let mut lot = |below: u32| {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) % below
    };
    let loaded: Vec<u32> = (1..0x100usize)
        .map(|n| unsafe { game_read::<u32>(OBJECT_KINDS_RVA + n * 4) })
        .filter(|&kind| kind != 0)
        .take(4)
        .collect();
    for case in 0..120 {
        let item = [1u32, ITEM][lot(2) as usize];
        let kind = match lot(7) {
            0 => 0x258,
            1 => 0x259,
            2 => 0x259 + lot(OWN),
            3 => 0xFFFF_FFFF,
            n => loaded.get(n as usize - 4).copied().unwrap_or(0x230 + n),
        };
        let own = [
            lot(0x1_0000),
            [u32::MAX, lot(0x10_0000)][lot(2) as usize],
            [u32::MAX, lot(0x1_0000)][lot(2) as usize],
        ];
        let (z, clock) = (lot(0x100_0000), lot(0x00FF_FFFF) << 4 | lot(16));
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.0.len()]);
            game_write(HIGHLIGHTED_RVA, item);
            game_write(Z_RVA, z);
            game_write(CLOCK_RVA, clock);
            game_write(KINDS_RVA + (item as usize - 1) * 4, kind);
            // The record the kind 0x259 + (lot) points at is not always the first:
            // the first one is set, and the others are the game's.
            game_write(OWN_RVA, own);
            DRAWN.lock().unwrap().clear();
            set_handler(drawing);
            let before = Regs::SENTINEL;
            let mut r = before;
            let kept = if rust {
                draw_highlighted(&r, &mut |r| {
                    act(0, r);
                    Ok(())
                })
                .is_ok()
            } else {
                call_game(&mut r, inner.addr());
                r.preserved_except(&before, 0)
            };
            (kept, capture(&regions), DRAWN.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 && theirs.2.len() == 1, || {
            format!(
                "case {case}: item {item:#x}, kind {kind:#x}, own {own:x?}, z {z:#x}, clock {clock:#x}: original {theirs:x?}; rust {mine:x?}"
            )
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
