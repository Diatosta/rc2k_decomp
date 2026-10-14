//! `gfx::load_picture_to_texture` (0x405313): a picture file into a system texture.
//!
//! In:  eax = the picture's path, ecx = the second picture's (for the alpha; 0 for
//!      none), ebx = the system texture
//! Out: carry clear; every register preserved
//!
//! With the record's +0x5C at 1 the second picture is read and kept 0x40000 bytes on
//! the picture's start, or +0x5C made 0 if there is none or it cannot be read
//! (`gfx/texture_upload.rs`); then the picture is read (`gfx::read_texture_picture`,
//! 256 x 256 of -1 if it cannot be) and `gfx::upload_texture_picture` puts it in.
//! Three call sites in `scene::load_texture_picture`, which calls the Rust version now.
//! Refuses a texture of 0x140 or more, a record without its format, and what its parts
//! refuse.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose upload notes what it is given, on
//! `gfx::load_picture`'s files.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::gfx::load_picture::{LoaderTest, TEST_PATHS};
use crate::gfx::read_texture_picture::read_texture_picture;
use crate::gfx::texture_upload::{
    RECORD_LEN, Record, SYSTEM_RECORDS_RVA, TEXTURES, keep_second_picture,
};
use crate::gfx::upload_texture_picture::{GameSurfaces, upload_texture_picture};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "gfx::load_picture_to_texture",
    rva(0x405313),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// `upload` gets the texture and what the reading left: the pixels, the width and the
/// height.
pub unsafe fn load_picture_to_texture(
    path: u32,
    second: u32,
    texture: u32,
    upload: &mut dyn FnMut(u32, u32, u32, u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let record = Record::of(texture)?;
    unsafe {
        keep_second_picture(&record, second)?;
        let (pixels, width, height, _) = read_texture_picture(path as *mut u8)?;
        upload(texture, pixels, width, height)
    }
}

/// With the Rust upload into the game's textures.
pub unsafe fn load_picture_to_texture_in_game(
    path: u32,
    second: u32,
    texture: u32,
) -> Result<(), Refused> {
    unsafe {
        load_picture_to_texture(
            path,
            second,
            texture,
            &mut |texture, pixels, width, height| {
                upload_texture_picture(texture, pixels, width, height, &mut GameSurfaces)
            },
        )
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { load_picture_to_texture_in_game(r.eax, r.ecx, r.ebx) } {
        Ok(()) => r.set_carry(false),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

const START: usize = 0x405313;
const LEN: usize = 0x43;
const ENTRY_BYTES: [u8; 5] = [0x60, 0x53, 0x6B, 0xF3, 0x78];
const CALL_SITES: [usize; 3] = [0x40532B, 0x405349, 0x40534F];
const UPLOAD_RVA: usize = rva(0x41A68B); // gfx::upload_texture_picture
const TEST_TEXTURE: u32 = TEXTURES - 1;

/// Each upload: eax, ebx, ecx, edx.
static CALLED: Mutex<Vec<[u32; 4]>> = Mutex::new(Vec::new());

fn handler(_place: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push([r.eax, r.ebx, r.ecx, r.edx]);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let calls: Vec<usize> = CALL_SITES.iter().map(|site| site - START).collect();
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(START)),
            LEN,
            &ENTRY_BYTES,
            &calls,
            &|callee| (callee == UPLOAD_RVA).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let loader = match LoaderTest::begin() {
        Ok(loader) => loader,
        Err(why) => return t.skip(why),
    };
    let record = game_addr(SYSTEM_RECORDS_RVA) as u32 + TEST_TEXTURE * RECORD_LEN as u32;
    let inputs: [Region; 1] = [(record, RECORD_LEN)];
    let saved = capture(&inputs);
    let description = [0u32; 8];
    let paths: Vec<Vec<u8>> = TEST_PATHS.iter().map(|p| [p, &[0][..]].concat()).collect();
    for keeps in [0u32, 1] {
        for (first, second) in [
            (0, None),
            (1, Some(0)),
            (0, Some(1)),
            (2, Some(3)),
            (3, Some(2)),
            (1, None),
        ] {
            let run = |rust: bool| unsafe {
                loader.prepare();
                let mut fields = [0u32; RECORD_LEN / 4];
                fields[1] = 0x100;
                fields[2] = 0x100;
                fields[3] = description.as_ptr() as u32;
                fields[0x5C / 4] = keeps;
                std::ptr::copy_nonoverlapping(
                    fields.as_ptr() as *const u8,
                    record as *mut u8,
                    RECORD_LEN,
                );
                let mut buffers = paths.clone();
                let path = buffers[first].as_mut_ptr() as u32;
                let second = second.map_or(0, |n: usize| buffers[n].as_mut_ptr() as u32);
                CALLED.lock().unwrap().clear();
                set_handler(handler);
                let ok = if rust {
                    load_picture_to_texture(path, second, TEST_TEXTURE, &mut |a, b, c, d| {
                        act(
                            0,
                            &mut Regs {
                                eax: a,
                                ebx: b,
                                ecx: c,
                                edx: d,
                                ..Regs::SENTINEL
                            },
                        );
                        Ok(())
                    })
                    .is_ok()
                } else {
                    let before = Regs {
                        eax: path,
                        ebx: TEST_TEXTURE,
                        ecx: second,
                        ..Regs::SENTINEL
                    };
                    let mut r = before;
                    call_game(&mut r, copy.addr());
                    r.preserved_except(&before, 0)
                };
                let paths_left: Vec<Vec<u8>> = buffers.clone();
                drop(buffers);
                (
                    ok,
                    CALLED.lock().unwrap().clone(),
                    capture(&inputs),
                    loader.observed(),
                    paths_left,
                )
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && theirs.0, || {
                let first_byte = theirs.3.iter().zip(&mine.3).position(|(a, b)| a != b);
                format!(
                    "keeps {keeps}, picture {first}, second {second:?}: ok {} / {}; uploads {:x?} / {:x?}; record same {}; first different byte of the picture {first_byte:?}; paths same {}",
                    theirs.0,
                    mine.0,
                    theirs.1,
                    mine.1,
                    theirs.2 == mine.2,
                    theirs.4 == mine.4
                )
            });
        }
    }
    restore(&inputs, &saved);
    loader.end();
    t.finish()
}
