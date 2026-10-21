//! `scene::draw_shown_object` (0x4503A5): draws the object `g_objectNumber` unless a
//! race's tests hide it.
//!
//! In:  nothing
//! Out: the registers as its parts leave them (after a drawing, `scene::draw_object`'s)
//!
//! `dword_62DA44` = 1 for the duration, 0 at the end.
//! In a race (`g_inRace` not 0) two tests, each of which can hide the object:
//! - `scene::object_hidden_by_kind` (0x4522EA over 0x452303, inlined): with
//!   `dword_71C3E0` not 0 or `dword_62389C` at 2, an object of kind 0x30 or 0x4E;
//! - `scene::object_hidden` (0x4503DD, inlined; leaves ebx = the object's number):
//!   objects 0x38 and 0x3A .. 0x3F are hidden with `dword_621480` set, with
//!   `dword_609330` at 0, object 0x3E with `dword_71B400` not 0 and object 0x3D with it
//!   not 1; with `dword_621480` set and `dword_635058` not 0 (eax), every object unless
//!   `dword_635058` - 1 is `dword_71B400`; and an object without faces
//!   (`g_objectFaceCounts`).
//!
//! Outside a race `scene::clear_kind_bytes` (0x46219C, inlined): the 0x100 dwords at
//! `dword_5CD688` = 0, then the seventh, eighth and tenth = 999 (eax = 0, ecx = 0,
//! edi = the table's end).
//! Then `scene::draw_object`.
//! One caller, `sub_451E3E`.
//! Refuses, in a race, an object's number of 0x100 or more (the original reads past
//! its tables).
//! Shadow mode runs the original alone and compares whether it drew with what the Rust
//! tests say (through `draw_object`'s hook, while that is on).
//! Self-test: a redirected copy whose drawing notes itself, with the original's own
//! tests, on settings drawn by lot.
//!
//! Converted 2026-10-21: the race's tests and the kind bytes through
//! `rc2k_core::scene::shown` (the variables read where the tests read them).

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::draw_object::{self, draw_object_in_game};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::draw_shown_object", rva(0x4503A5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const DRAWING_RVA: usize = rva(0x62DA44); // dword_62DA44
const IN_RACE_RVA: usize = rva(0x5C5CEC); // g_inRace
const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
const KIND_SWITCH_RVA: usize = rva(0x71C3E0); // dword_71C3E0
const KIND_MODE_RVA: usize = rva(0x62389C); // dword_62389C
const KINDS_RVA: usize = rva(0x5CD27C); // g_objectKinds
const OTHER_WAY_RVA: usize = rva(0x621480); // dword_621480
const SHOWN_RVA: usize = rva(0x609330); // dword_609330
const WHICH_RVA: usize = rva(0x71B400); // dword_71B400
const ONLY_RVA: usize = rva(0x635058); // dword_635058
const FACE_COUNTS_RVA: usize = rva(0x632F70); // dword_632F70
const KIND_BYTES_RVA: usize = rva(0x5CD688); // dword_5CD688
use rc2k_core::scene::shown::OBJECTS;

// ---- Rust version ----

/// The game's variables for the race's tests.
struct Game;

impl rc2k_core::scene::shown::RaceObject for Game {
    fn object(&self) -> u32 {
        unsafe { game_read(OBJECT_RVA) }
    }
    fn kind_switch(&self) -> u32 {
        unsafe { game_read(KIND_SWITCH_RVA) }
    }
    fn kind_mode(&self) -> u32 {
        unsafe { game_read(KIND_MODE_RVA) }
    }
    fn kind(&self, object: u32) -> u32 {
        unsafe { game_read(KINDS_RVA + object as usize * 4) }
    }
    fn other_way(&self) -> u32 {
        unsafe { game_read(OTHER_WAY_RVA) }
    }
    fn shown(&self) -> u32 {
        unsafe { game_read(SHOWN_RVA) }
    }
    fn which(&self) -> u32 {
        unsafe { game_read(WHICH_RVA) }
    }
    fn only(&self) -> u32 {
        unsafe { game_read(ONLY_RVA) }
    }
    fn faces(&self, object: u32) -> u32 {
        unsafe { game_read(FACE_COUNTS_RVA + object as usize * 4) }
    }
}

/// The race's two tests: whether the object is drawn (its number checked by the caller).
/// Writes nothing but the registers the tests leave.
// Converted: `rc2k_core::scene::shown::race_shown`
fn shown(r: &mut Regs) -> bool {
    let s = rc2k_core::scene::shown::race_shown(&Game);
    r.ebx = s.ebx.unwrap_or(r.ebx);
    r.eax = s.eax.unwrap_or(r.eax);
    s.drawn
}

/// `draw`: what stands for `scene::draw_object`. Says whether the object was drawn.
pub unsafe fn draw_shown_object(
    r: &mut Regs,
    draw: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
) -> Result<bool, Refused> {
    unsafe {
        let race = game_read::<u32>(IN_RACE_RVA) != 0;
        if race && game_read::<u32>(OBJECT_RVA) >= OBJECTS {
            return Err(BadInput("an object's number past the tables").into());
        }
        game_write(DRAWING_RVA, 1u32);
        let show = if race {
            shown(r)
        } else {
            game_write(KIND_BYTES_RVA, rc2k_core::scene::shown::kind_bytes());
            r.eax = 0;
            r.ecx = 0;
            r.edi = (game_addr(KIND_BYTES_RVA) + 0x400) as u32;
            true
        };
        if show {
            draw(r)?;
        }
        game_write(DRAWING_RVA, 0u32);
        Ok(show)
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        let race = unsafe { game_read::<u32>(IN_RACE_RVA) } != 0;
        let object: u32 = unsafe { game_read(OBJECT_RVA) };
        let said = match race {
            false => Some(true),
            true if object < OBJECTS => Some(shown(&mut { *r })),
            true => None,
        };
        let before = draw_object::ENTERED.load(Ordering::Relaxed);
        unsafe { call_original(r) };
        let drew = draw_object::ENTERED.load(Ordering::Relaxed) != before;
        if draw_object::HOOK.is_enabled() && said.is_some_and(|said| said != drew) {
            HOOK.mismatch(|| {
                format!("object {object:#x}, in a race {race}: original drew {drew}, rust {said:?}")
            });
        }
        return;
    }
    let entry = *r;
    if let Err(e) = unsafe { draw_shown_object(r, &mut |r| draw_object_in_game(r)) } {
        *r = entry;
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0x38;
const ENTRY_BYTES: [u8; 10] = [0xC7, 0x05, 0x44, 0xDA, 0x62, 0x00, 1, 0, 0, 0];
const CALLS: [usize; 4] = [0x13, 0x1A, 0x23, 0x28];

/// Each drawing: the switch by then and eax, ebx, ecx, edi as it was entered with.
static CALLED: Mutex<Vec<[u32; 5]>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push([
        unsafe { game_read(DRAWING_RVA) },
        r.eax,
        r.ebx,
        r.ecx,
        r.edi,
    ]);
    r.eax = r.eax.wrapping_mul(5).wrapping_add(1);
    r.ebp ^= 0x0F0F_0F0F;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x4503A5)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == rva(0x45045D)).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions: [Region; 2] = [at(DRAWING_RVA), (game_addr(KIND_BYTES_RVA) as u32, 0x400)];
    let inputs: [Region; 10] = [
        at(IN_RACE_RVA),
        at(OBJECT_RVA),
        at(KIND_SWITCH_RVA),
        at(KIND_MODE_RVA),
        at(OTHER_WAY_RVA),
        at(SHOWN_RVA),
        at(WHICH_RVA),
        at(ONLY_RVA),
        (game_addr(KINDS_RVA) as u32, OBJECTS as usize * 4),
        (game_addr(FACE_COUNTS_RVA) as u32, OBJECTS as usize * 4),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let mut seed = 0x2545_F491u32;
    let mut lot = |below: u32| {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) % below
    };
    let mut drawn = 0;
    for case in 0..400 {
        let object = [
            0x37u32, 0x38, 0x39, 0x3A, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0xF2,
        ][lot(10) as usize];
        let race = [1u32, 1, 1, 0x100, 0][lot(5) as usize];
        let kind_switch = [0u32, 0, 1][lot(3) as usize];
        let kind_mode = lot(4);
        let kind = [0x30u32, 0x4E, 5, 0x4F][lot(4) as usize];
        let other_way = [0u32, 0, 2][lot(3) as usize];
        let shown_switch = [1u32, 1, 0][lot(3) as usize];
        let which = lot(3);
        let only = lot(4);
        let faces = [3u32, 1, 0, 0x8000_0000][lot(4) as usize];
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.0.len()]);
            game_write(IN_RACE_RVA, race);
            game_write(OBJECT_RVA, object);
            game_write(KIND_SWITCH_RVA, kind_switch);
            game_write(KIND_MODE_RVA, kind_mode);
            game_write(KINDS_RVA + object as usize * 4, kind);
            game_write(OTHER_WAY_RVA, other_way);
            game_write(SHOWN_RVA, shown_switch);
            game_write(WHICH_RVA, which);
            game_write(ONLY_RVA, only);
            game_write(FACE_COUNTS_RVA + object as usize * 4, faces);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let mut r = Regs::SENTINEL;
            if rust {
                let done = draw_shown_object(&mut r, &mut |r| {
                    act(0, r);
                    Ok(())
                });
                if done.is_err() {
                    r = Regs::default();
                }
            } else {
                call_game(&mut r, copy.addr());
            }
            (
                [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
                capture(&regions),
                CALLED.lock().unwrap().clone(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        drawn += theirs.2.len();
        t.check(theirs == mine, || {
            format!(
                "case {case}: object {object:#x}, race {race:#x}, kind {kind:#x} ({kind_switch}, {kind_mode}), other way {other_way}, shown {shown_switch}, which {which}, only {only}, faces {faces:#x}: original regs {:x?} drew {:x?}; rust regs {:x?} drew {:x?}; memory same {}",
                theirs.0,
                theirs.2,
                mine.0,
                mine.2,
                theirs.1 == mine.1,
            )
        });
    }
    // Both ways must have been taken, or the lot was a poor one.
    t.check(drawn > 50 && drawn < 350, || {
        format!("the cases drew {drawn} times of 400")
    });
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
