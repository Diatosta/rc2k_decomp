//! `scene::draw_scene_object` (0x451E3E): chooses the set of textures for the object
//! `g_objectNumber` and draws it the way the scene's mode asks for.
//!
//! In:  esi, passed on to `scene::draw_described_object`
//! Out: the registers as its parts leave them; before the drawing eax = what was last
//!      looked at, ebx = the object's number, edx = the car's remainder if a car was
//!      looked at
//!
//! An object's number of 0x100 or more is fatal error 0x80600A01.
//! `dword_639954` (added to every face's texture number) = 0; then 0x108 for, outside
//! a race, an object of kind 0x23A, and in a race for objects 0x50 .. 0xEF (the cars,
//! 0x14 objects each) whose car's block has 0x21 at +0x24.
//! Nothing more for an object without faces (`g_objectFaceCounts`). Else by
//! `dword_6365C4`: 0 `scene::draw_shown_object`; 1 and 2 `scene::draw_described_object` (it
//! takes an object's description in esi and comes back here); anything else nothing.
//! Twelve callers.
//! Refuses a car without its block.
//! Shadow mode runs the original alone and, for mode 0, checks the texture set it
//! chose against the Rust arithmetic.
//! Self-test: a redirected copy whose three calls note themselves.
//!
//! Converted 2026-10-21: the texture set and the way on through
//! `rc2k_core::scene::shown`.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::draw_object::draw_object_in_game;
use crate::scene::draw_shown_object::draw_shown_object;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::draw_scene_object", rva(0x451E3E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
const TEXTURE_SET_RVA: usize = rva(0x639954); // dword_639954
const IN_RACE_RVA: usize = rva(0x5C5CEC); // g_inRace
const KINDS_RVA: usize = rva(0x5CD27C); // g_objectKinds
const BLOCKS_RVA: usize = rva(0x71B3B8); // g_carBlocks
const FACE_COUNTS_RVA: usize = rva(0x632F70); // dword_632F70
const MODE_RVA: usize = rva(0x6365C4); // dword_6365C4
const DESCRIBED_RVA: usize = rva(0x451B61); // scene::draw_described_object

const TOO_HIGH: FatalError = FatalError {
    code: 0x8060_0A01,
    message: "object number too high",
};

use rc2k_core::scene::shown::OBJECTS;
/// The two ways on from here.
pub use rc2k_core::scene::shown::Way;

// ---- Rust version ----

/// The texture set and the registers the choosing leaves. Writes only `r`.
// Converted: `rc2k_core::scene::shown::texture_set`
unsafe fn texture_set(r: &mut Regs) -> Result<u32, Refused> {
    use rc2k_core::scene::shown::SetError;
    unsafe {
        let object: u32 = game_read(OBJECT_RVA);
        r.ebx = object;
        let t = rc2k_core::scene::shown::texture_set(
            object,
            game_read::<u32>(IN_RACE_RVA) != 0,
            r.eax,
            |object| game_read(KINDS_RVA + object as usize * 4),
            |car| {
                let block: u32 = game_read(BLOCKS_RVA + car as usize * 4);
                (block != 0).then(|| ((block + 0x24) as *const u32).read_unaligned())
            },
        )
        .map_err(|e| -> Refused {
            match e {
                SetError::TooHigh => TOO_HIGH.into(),
                SetError::NoBlock => BadInput("a car without its block").into(),
            }
        })?;
        r.eax = t.eax;
        r.edx = t.edx.unwrap_or(r.edx);
        Ok(t.set)
    }
}

/// `draw`: what stands for the two calls.
// Converted: `rc2k_core::scene::shown::way`
pub unsafe fn draw_scene_object(
    r: &mut Regs,
    draw: &mut dyn FnMut(Way, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let mut left = *r;
        let set = texture_set(&mut left)?;
        *r = left;
        game_write(TEXTURE_SET_RVA, set);
        let faces: u32 = game_read(FACE_COUNTS_RVA + r.ebx as usize * 4);
        match rc2k_core::scene::shown::way(faces, game_read(MODE_RVA)) {
            Some(way) => draw(way, r),
            None => Ok(()),
        }
    }
}

/// `draw_scene_object` with the game's parts (`scene::draw_shown_object` and its drawing;
/// `scene::draw_described_object`), for Rust callers.
pub unsafe fn draw_scene_object_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        draw_scene_object(r, &mut |way, r| match way {
            Way::Shown => draw_shown_object(r, &mut |r| draw_object_in_game(r)).map(|_| ()),
            Way::Described => crate::scene::draw_described_object::draw_described_object(
                r,
                &mut crate::scene::draw_described_object::described_part,
            ),
        })
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        // Mode 0 does not come back here, so the set is still this call's afterwards.
        let said = (unsafe { game_read::<u32>(MODE_RVA) } == 0)
            .then(|| unsafe { texture_set(&mut { *r }) }.ok())
            .flatten();
        let object: u32 = unsafe { game_read(OBJECT_RVA) };
        unsafe { call_original(r) };
        let set: u32 = unsafe { game_read(TEXTURE_SET_RVA) };
        if said.is_some_and(|said| said != set) {
            HOOK.mismatch(|| {
                format!("object {object:#x}: texture set original {set:#x}, rust {said:x?}")
            });
        }
        return;
    }
    let entry = *r;
    let done = unsafe { draw_scene_object_in_game(r) };
    if let Err(e) = done {
        *r = entry;
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0xAF;
const ENTRY_BYTES: [u8; 6] = [0x8B, 0x1D, 0xFC, 0x14, 0x62, 0x00];
const CALLS: [usize; 3] = [0x9D, 0xA3, 0xA9];

/// Each call: its place, the texture set by then and eax, ebx, edx, esi as it was
/// entered with.
static CALLED: Mutex<Vec<[u32; 6]>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push([
        place,
        unsafe { game_read(TEXTURE_SET_RVA) },
        r.eax,
        r.ebx,
        r.edx,
        r.esi,
    ]);
    r.eax ^= 0x5A5A_0000 + place;
    r.ecx = r.ecx.wrapping_add(place + 1);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The jump to the fatal stub is left as it is: no case takes it.
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x451E3E)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                if callee == rva(0x4503A5) {
                    Some(stand_in(0))
                } else if callee == DESCRIBED_RVA {
                    Some(stand_in(1))
                } else {
                    None
                }
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions: [Region; 1] = [at(TEXTURE_SET_RVA)];
    let inputs: [Region; 6] = [
        at(OBJECT_RVA),
        at(IN_RACE_RVA),
        at(MODE_RVA),
        (game_addr(KINDS_RVA) as u32, OBJECTS as usize * 4),
        (game_addr(FACE_COUNTS_RVA) as u32, OBJECTS as usize * 4),
        (game_addr(BLOCKS_RVA) as u32, 8 * 4),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let mut block = [0xAAAA_AAAAu32; 16];
    let mut calls = [0usize; 2];
    for object in [0u32, 1, 0x4F, 0x50, 0x63, 0x64, 0x77, 0xEF, 0xF0, 0xFF] {
        for race in [0u32, 1] {
            for (kind, mark) in [(0x23Au32, 0x21u32), (5, 0x20), (0x21, 0x23A)] {
                for faces in [0u32, 2] {
                    for mode in [0u32, 1, 2, 3] {
                        block[9] = mark;
                        let block_at = block.as_ptr() as u32;
                        let run = |rust: bool| unsafe {
                            game_write(TEXTURE_SET_RVA, 0xAAAA_AAAAu32);
                            game_write(OBJECT_RVA, object);
                            game_write(IN_RACE_RVA, race);
                            game_write(MODE_RVA, mode);
                            game_write(KINDS_RVA + object as usize * 4, kind);
                            game_write(FACE_COUNTS_RVA + object as usize * 4, faces);
                            game_write(BLOCKS_RVA, [block_at; 8]);
                            CALLED.lock().unwrap().clear();
                            set_handler(handler);
                            let mut r = Regs::SENTINEL;
                            if rust {
                                let done = draw_scene_object(&mut r, &mut |way, r| {
                                    act(way as u32, r);
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
                                game_read::<u32>(TEXTURE_SET_RVA),
                                CALLED.lock().unwrap().clone(),
                            )
                        };
                        let (theirs, mine) = (run(false), run(true));
                        for call in &theirs.2 {
                            calls[call[0] as usize] += 1;
                        }
                        t.check(theirs == mine, || {
                            format!(
                                "object {object:#x}, race {race}, kind {kind:#x}, mark {mark:#x}, faces {faces}, mode {mode}: original {theirs:x?}; rust {mine:x?}"
                            )
                        });
                    }
                }
            }
        }
    }
    // The fatal error, which the copy cannot be asked for.
    let too_high = unsafe {
        game_write(OBJECT_RVA, OBJECTS);
        let mut r = Regs::SENTINEL;
        draw_scene_object(&mut r, &mut |_, _| Ok(()))
    };
    t.check(
        matches!(&too_high, Err(Refused::Fatal(e)) if e.code == TOO_HIGH.code),
        || format!("object 0x100: rust {too_high:x?}"),
    );
    t.check(calls[0] > 0 && calls[1] > 0, || {
        format!("the cases reached the two calls {calls:?} times")
    });
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
