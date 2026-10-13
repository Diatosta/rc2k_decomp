//! `scene::draw_object_faces` (0x450A4B): draws the faces of an object that were listed
//! for drawing.
//!
//! In:  nothing
//! Out: nothing changed with an empty list; else eax, ecx, esi and ebp are the loop's
//!      leftovers (the last face's number in the table, the count, the last list
//!      entry, the last face); everything else preserved
//!
//! The list: `dword_62DB58` entries of 0x10 bytes at `dword_62DB5C`: +0 the face's
//! number within the object, +0xC how see-through it is (to `g_objectSeeThrough`).
//! For each: the number to `dword_623620`; the face is entry number + `dword_6350BC`
//! of the table `dword_9A3730`; from the face's head the first texture (the word +4
//! plus `dword_639954`, with the top bit set), its kind (byte +6, `g_faceKind`) and
//! byte +0xB (`dword_6D18D0`); the polygon's colour is white, or with a byte +0xA
//! not 0 that byte in all three places; `scene::draw_object_face`; the colour white
//! again. At the end `dword_624204` = 0.
//! One caller, `sub_45045D`.
//! Refuses a face's number past the table and a face that is not there.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose call notes the face and what was set for it.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, EBP, ECX, ESI, RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write,
    hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::draw_object_face::draw_object_face_in_game;
use crate::scene::polygon::dword;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::draw_object_faces", rva(0x450A4B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const LISTED_RVA: usize = rva(0x62DB58); // dword_62DB58
pub const LIST_RVA: usize = rva(0x62DB5C); // dword_62DB5C
const LIST_ENTRY_LEN: usize = 0x10;
const FACE_NUMBER_RVA: usize = rva(0x623620); // dword_623620
const FIRST_FACE_RVA: usize = rva(0x6350BC); // dword_6350BC
const FACES_RVA: usize = rva(0x9A3730); // dword_9A3730
const FACES: u32 = 0xC000;
const SEE_THROUGH_RVA: usize = rva(0x639970); // g_objectSeeThrough
const TEXTURE_A_RVA: usize = rva(0x6240BC); // dword_6240BC
const FIRST_TEXTURE_RVA: usize = rva(0x639954); // dword_639954
const FACE_KIND_RVA: usize = rva(0x6D18CC); // g_faceKind
const FACE_UNLIT_RVA: usize = rva(0x6D18D0); // dword_6D18D0
const COLOUR_RVA: usize = rva(0x6240CC); // g_polygonColour
const DONE_RVA: usize = rva(0x624204); // dword_624204

// ---- Rust version ----

pub unsafe fn draw_object_faces(
    draw: &mut dyn FnMut(usize) -> Result<(), Refused>,
) -> Result<Left, Refused> {
    unsafe {
        let listed: u32 = game_read(LISTED_RVA);
        if listed == 0 {
            return Ok(Left::nothing());
        }
        let first: u32 = game_read(FIRST_FACE_RVA);
        // Every face is looked up before anything is written.
        let mut faces = Vec::with_capacity(listed as usize);
        for n in 0..listed as usize {
            let entry = game_addr(LIST_RVA) + n * LIST_ENTRY_LEN;
            let number = dword(entry).wrapping_add(first);
            if number >= FACES {
                return Err(BadInput("a face's number past the table").into());
            }
            let face: u32 = game_read(FACES_RVA + number as usize * 4);
            if face == 0 {
                return Err(BadInput("a face that is not there").into());
            }
            faces.push((entry, number, face as usize));
        }
        for &(entry, _, face) in &faces {
            game_write(SEE_THROUGH_RVA, dword(entry + 0xC));
            game_write(FACE_NUMBER_RVA, dword(entry));
            let texture = ((face + 4) as *const u16).read_unaligned() as u32;
            game_write(
                TEXTURE_A_RVA,
                texture.wrapping_add(game_read(FIRST_TEXTURE_RVA)) | 0x8000_0000,
            );
            game_write(FACE_KIND_RVA, ((face + 6) as *const u8).read() as u32);
            game_write(FACE_UNLIT_RVA, ((face + 0xB) as *const u8).read() as u32);
            let shade = ((face + 0xA) as *const u8).read() as u32;
            game_write(
                COLOUR_RVA,
                if shade == 0 {
                    0x00FF_FFFF
                } else {
                    0x0001_0101 * shade
                },
            );
            draw(face)?;
            game_write(COLOUR_RVA, 0x00FF_FFFFu32);
        }
        game_write(DONE_RVA, 0u32);
        let &(entry, number, face) = faces.last().expect("the list is not empty");
        Ok(Left::nothing()
            .with(EAX, number)
            .with(ECX, listed)
            .with(ESI, entry as u32)
            .with(EBP, face as u32))
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    match unsafe { draw_object_faces(&mut |face| draw_object_face_in_game(face, &entry)) } {
        Ok(left) => left.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

const LEN: usize = 0xA4;
const ENTRY_BYTES: [u8; 7] = [0x83, 0x3D, 0x58, 0xDB, 0x62, 0x00, 0x00];
const CALLS: [usize; 1] = [0x7B];

fn regions() -> Vec<Region> {
    [
        SEE_THROUGH_RVA,
        FACE_NUMBER_RVA,
        TEXTURE_A_RVA,
        FACE_KIND_RVA,
        FACE_UNLIT_RVA,
        COLOUR_RVA,
        DONE_RVA,
    ]
    .map(at)
    .to_vec()
}

/// Each call: the face and what was set for it.
static CALLED: Mutex<Vec<(u32, Vec<u8>)>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push((r.ebp, capture(&regions())));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x450A4B)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == rva(0x450BDA)).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    const MOST: usize = 4;
    const FIRST: u32 = 0x100;
    let regions = regions();
    let inputs = [
        at(LISTED_RVA),
        (game_addr(LIST_RVA) as u32, MOST * LIST_ENTRY_LEN),
        at(FIRST_FACE_RVA),
        at(FIRST_TEXTURE_RVA),
        (game_addr(FACES_RVA + FIRST as usize * 4) as u32, 8 * 4),
    ];
    let saved = (capture(&regions), capture(&inputs));
    // Faces' heads: the word +4, the bytes +6, +0xA, +0xB.
    let heads: [[u8; 12]; 4] = [
        [0, 0, 0, 0, 0x34, 0x12, 0x1C, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0xFF, 0xFF, 5, 0, 0, 0, 0x80, 1],
        [0, 0, 0, 0, 0, 0, 0xB, 0, 0, 0, 0xFF, 0],
        [0, 0, 0, 0, 7, 0, 3, 0, 0, 0, 1, 0xFE],
    ];
    for listed in 0..=MOST as u32 {
        for first_texture in [0u32, 0x200, 0xFFFF_FFF0] {
            let run = |rust: bool| unsafe {
                restore(&regions, &vec![0xAA; saved.0.len()]);
                game_write(LISTED_RVA, listed);
                game_write(FIRST_FACE_RVA, FIRST);
                game_write(FIRST_TEXTURE_RVA, first_texture);
                for n in 0..MOST {
                    // The list names the faces out of order.
                    let number = [2u32, 0, 7, 5][n];
                    let entry = game_addr(LIST_RVA) + n * LIST_ENTRY_LEN;
                    (entry as *mut [u32; 4]).write_unaligned([
                        number,
                        0xAAAA_AAAA,
                        0xAAAA_AAAA,
                        0x100 - n as u32 * 0x55,
                    ]);
                    game_write(
                        FACES_RVA + (FIRST + number) as usize * 4,
                        heads[n].as_ptr() as u32,
                    );
                }
                CALLED.lock().unwrap().clear();
                set_handler(handler);
                let before = Regs::SENTINEL;
                let left = if rust {
                    draw_object_faces(&mut |face| {
                        act(
                            0,
                            &mut Regs {
                                ebp: face as u32,
                                ..Regs::default()
                            },
                        );
                        Ok(())
                    })
                    .ok()
                    .map(|left| {
                        let mut r = before;
                        left.apply(&mut r);
                        (r.eax, r.ecx, r.esi, r.ebp)
                    })
                } else {
                    let mut ro = before;
                    call_game(&mut ro, copy.addr());
                    ro.preserved_except(&before, EAX | ECX | ESI | EBP)
                        .then_some((ro.eax, ro.ecx, ro.esi, ro.ebp))
                };
                (left, capture(&regions), CALLED.lock().unwrap().clone())
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && theirs.2.len() == listed as usize, || {
                format!(
                    "{listed} listed, first texture {first_texture:#x}: original {theirs:x?}; rust {mine:x?}"
                )
            });
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
