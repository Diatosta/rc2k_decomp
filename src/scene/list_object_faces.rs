//! `scene::list_object_faces` (0x45048C): goes through an object's faces and lists the
//! ones to draw.
//!
//! In:  nothing
//! Out: eax = the last face's number in the table, ecx = the number of faces, ebp = the
//!      last face; everything else preserved
//!
//! The list emptied (`g_listedFaceCount` = 0), `dword_62361C` = -1; then for each of
//! the object's `dword_6350C0` faces, from `dword_6350BC` on in `g_faces`: its number
//! within the object to `dword_623620` and `scene::list_object_face`.
//! One caller, `sub_45045D`.
//! Refuses an object of no faces (the original goes through 2^32) or of more than the
//! list holds, a face's number past the table and a face that is not there.
//! Shadow mode runs the original alone (each face is compared in
//! `scene::list_object_face`'s hook).
//! Self-test: a redirected copy whose call notes the face and its number.

use crate::errors::Refused;
use crate::framework::{
    EAX, EBP, ECX, RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm,
    rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::draw_object_faces::LISTED_RVA;
use crate::scene::list_object_face::list_object_face;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::list_object_faces", rva(0x45048C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const MARK_RVA: usize = rva(0x62361C); // dword_62361C
const FACE_NUMBER_RVA: usize = rva(0x623620); // dword_623620
const FIRST_FACE_RVA: usize = rva(0x6350BC); // dword_6350BC
const FACE_COUNT_RVA: usize = rva(0x6350C0); // dword_6350C0
const FACES_RVA: usize = rva(0x9A3730); // g_faces

// ---- Rust version ----

// Converted: `rc2k_core::scene::objects::object_faces`
pub unsafe fn list_object_faces(
    list: &mut dyn FnMut(usize) -> Result<(), Refused>,
) -> Result<Left, Refused> {
    unsafe {
        let (first, count): (u32, u32) = (game_read(FIRST_FACE_RVA), game_read(FACE_COUNT_RVA));
        // Every face is looked up before anything is written.
        let faces = rc2k_core::scene::objects::object_faces(first, count, |number| {
            game_read(FACES_RVA + number as usize * 4)
        })?;
        game_write(LISTED_RVA, 0u32);
        game_write(MARK_RVA, u32::MAX);
        for (n, &face) in faces.iter().enumerate() {
            game_write(FACE_NUMBER_RVA, n as u32);
            list(face as usize)?;
        }
        Ok(Left::nothing()
            .with(EAX, (count - 1).wrapping_add(first))
            .with(ECX, count)
            .with(EBP, faces[count as usize - 1]))
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { list_object_faces(&mut |face| list_object_face(face)) } {
        Ok(left) => left.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { list_object_faces(&mut |face| list_object_face(face)) }?;
    left.apply(r);
    Ok(())
}

// ---- self-test ----

const LEN: usize = 0x3F;
const ENTRY_BYTES: [u8; 10] = [0xC7, 0x05, 0x58, 0xDB, 0x62, 0x00, 0, 0, 0, 0];
const CALLS: [usize; 1] = [0x2F];

/// Each call: the face, its number, the list's length and the mark by then.
static CALLED: Mutex<Vec<[u32; 4]>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    unsafe {
        CALLED.lock().unwrap().push([
            r.ebp,
            game_read(FACE_NUMBER_RVA),
            game_read(LISTED_RVA),
            game_read(MARK_RVA),
        ]);
        // As the real one does for some faces.
        if r.ebp & 1 == 1 {
            game_write(LISTED_RVA, game_read::<u32>(LISTED_RVA) + 1);
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x45048C)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == rva(0x4504CB)).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    const FIRST: u32 = 0x200;
    const MOST: u32 = 6;
    let regions = [at(LISTED_RVA), at(MARK_RVA), at(FACE_NUMBER_RVA)];
    let inputs = [
        at(FIRST_FACE_RVA),
        at(FACE_COUNT_RVA),
        (
            game_addr(FACES_RVA + FIRST as usize * 4) as u32,
            MOST as usize * 4,
        ),
    ];
    let saved = (capture(&regions), capture(&inputs));
    for count in 1..=MOST {
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.0.len()]);
            game_write(FIRST_FACE_RVA, FIRST);
            game_write(FACE_COUNT_RVA, count);
            for n in 0..MOST {
                // Not addresses the stand-in reads through: only told apart.
                game_write(
                    FACES_RVA + (FIRST + n) as usize * 4,
                    0x7000_0000 + n * 7 + count,
                );
            }
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let before = Regs::SENTINEL;
            let left = if rust {
                list_object_faces(&mut |face| {
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
                    (r.eax, r.ecx, r.ebp)
                })
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                ro.preserved_except(&before, EAX | ECX | EBP)
                    .then_some((ro.eax, ro.ecx, ro.ebp))
            };
            (left, capture(&regions), CALLED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.2.len() == count as usize, || {
            format!("{count} faces: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
