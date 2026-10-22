//! `scene::upload_plate` (0x41B2F2): one number plate painted into a texture.
//!
//! In:  eax, ecx = the plate sheet's two pictures (the second for the alpha), ebx = the
//!      system texture, edx, esi = where on the sheet the plate is, edi, ebp = where on
//!      the texture it goes
//! Out: every register preserved
//!
//! - ebx, edi, ebp, edx, esi to `dword_5BE8F1`, `_8F5`, `_8F9`, `_8FD`, `_901`, the
//!   plate's size 0x52 x 0x2A to `dword_5BE905`, `_909`.
//! - With the texture's record (0x78 bytes at `unk_5B521C`) +0x5C at 1: the second
//!   picture read (`gfx::read_texture_picture`) and copied 0x40000 bytes on, or, if
//!   there is none or it cannot be read, +0x5C made 0.
//! - The first picture read; `least_colour` of the texture's format (record +0xC, +8);
//!   `alpha_from_kept` or `key_texture_colours` by +0x5C; `grade_texture`.
//! - The picture and the texture must be the same number of halvings from 256 (the
//!   table `dword_5ABA98`: 256, 128, 64, 32, and 1024 as 4), in width and in height;
//!   a size not in it, or two different numbers, is a fatal error (0x80A18761 ..
//!   0x80A18767). The picture is halved that many times (`halve_texture`) and the six
//!   places and sizes stored are shifted right by it.
//! - The texture locked (`sub_41AA4B`, which makes it `g_surface`); if it locked, the
//!   plate's piece blitted (`gfx::blit_components`, the picture's row bytes being its
//!   height times 4: the original's, a square sheet makes it right) and the texture
//!   unlocked (`sub_41AB02`). Those two are Direct3D and stay the game's.
//!
//! Two call sites in `scene::load_number_plates`.
//! Refuses a texture of 0x140 or more, a record without its format, and what the
//! pixel functions refuse; a refusal after the pictures are read hands the original
//! the call from the start.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose lock, blit and unlock note what they are given,
//! on sheets of the test's in a texture record of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::gfx::blit_components::{blit_components, game_target};
use crate::gfx::load_picture::LoaderTest;
use crate::gfx::pixel_format::TEST_FORMATS;
use crate::gfx::read_texture_picture::read_texture_picture;
use crate::gfx::texture_pixels::{
    GRADES_RVA, READING_RVA, SATURATION_RVA, SCALE_RVA, halvable, halve_texture,
};
use crate::gfx::texture_upload::{
    RECORD_LEN, Record, SYSTEM_RECORDS_RVA, TEXTURES, halvings_apart, keep_second_picture,
    ready_picture,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::upload_plate", rva(0x41B2F2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// `stru_5BE8F1` (`PlateUpload` in IDA, unaligned): what the upload keeps while it works.
const TEXTURE_RVA: usize = rva(0x5BE8F1);
const STORED_LEN: usize = std::mem::size_of::<PlateUpload>();

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct PlateUpload {
    pub texture: u32,
    /// Where on the texture the plate goes.
    pub target_x: u32,
    pub target_y: u32,
    /// The plate's piece of the sheet: x, y, width, height (`blit_components`' rectangle).
    pub piece: [u32; 4],
}

const _: () = assert!(STORED_LEN == 0x1C);

/// The piece's place: what `blit_components` gets in ebx.
fn piece_address() -> u32 {
    (game_addr(TEXTURE_RVA) + std::mem::offset_of!(PlateUpload, piece)) as u32
}
const LOCK_RVA: usize = rva(0x41AA4B); // sub_41AA4B
const UNLOCK_RVA: usize = rva(0x41AB02); // sub_41AB02

/// What it asks the platform for.
pub enum Call {
    /// Returns whether the texture was locked (and made the surface).
    Lock(u32),
    /// `gfx::blit_components` with these registers.
    Blit(Regs),
    Unlock(u32),
}

// ---- Rust version ----

pub unsafe fn upload_plate(r: &Regs, call: &mut dyn FnMut(Call) -> bool) -> Result<(), Refused> {
    unsafe {
        let texture = r.ebx;
        let record = Record::of(texture)?;
        game_write(
            TEXTURE_RVA,
            PlateUpload {
                texture,
                target_x: r.edi,
                target_y: r.ebp,
                piece: [r.edx, r.esi, 0x52, 0x2A],
            },
        );
        keep_second_picture(&record, r.ecx)?;
        let (pixels, width, height, _) = read_texture_picture(r.eax as *mut u8)?;
        ready_picture(&record, pixels, width, height)?;
        let across = halvings_apart(
            &record,
            width,
            height,
            [
                0x80A1_8761,
                0x80A1_8762,
                0x80A1_8764,
                0x80A1_8765,
                0x80A1_8767,
            ],
        )?;
        if !halvable(width, height, across) {
            return Err(BadInput("a picture that halves to nothing").into());
        }
        let (_, height) = halve_texture(pixels as *mut u32, width, height, across);
        let mut stored: PlateUpload = game_read(TEXTURE_RVA);
        let shift = across & 31;
        stored.target_x >>= shift;
        stored.target_y >>= shift;
        stored.piece = stored.piece.map(|value| value >> shift);
        game_write(TEXTURE_RVA, stored);
        if call(Call::Lock(texture)) {
            call(Call::Blit(Regs {
                eax: height.wrapping_mul(4),
                ebx: piece_address(),
                ecx: stored.target_x,
                edx: stored.target_y,
                esi: pixels,
                ..*r
            }));
            call(Call::Unlock(texture));
        }
        Ok(())
    }
}

/// With the game's lock and unlock.
pub unsafe fn upload_plate_in_game(r: &Regs) -> Result<(), Refused> {
    let mut blitted = Ok(());
    let done = unsafe {
        upload_plate(r, &mut |call| match call {
            Call::Lock(texture) => {
                let mut regs = Regs { eax: texture, ..*r };
                crate::calls::call(LOCK_RVA, &mut regs);
                !regs.carry()
            }
            Call::Blit(regs) => {
                blitted = blit_components(&regs, &game_target());
                true
            }
            Call::Unlock(texture) => {
                let mut regs = Regs { eax: texture, ..*r };
                crate::calls::call(UNLOCK_RVA, &mut regs);
                true
            }
        })
    };
    done.and(blitted)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { upload_plate_in_game(r) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const START: usize = 0x41B2F2;
const LEN: usize = 0x18E;
const ENTRY_BYTES: [u8; 7] = [0x60, 0x89, 0x1D, 0xF1, 0xE8, 0x5B, 0x00];
const CALL_SITES: [usize; 14] = [
    0x41B33C, 0x41B35A, 0x41B36F, 0x41B37A, 0x41B381, 0x41B386, 0x41B39A, 0x41B3B5, 0x41B3DD,
    0x41B3F8, 0x41B41D, 0x41B44D, 0x41B46F, 0x41B479,
];
const BLIT_RVA: usize = rva(0x4065EE); // gfx::blit_components
const TEST_TEXTURE: u32 = TEXTURES - 1;
const SHEET: &str = "var\\rc2k_hook_plate_a.pcx";
const KEPT_SHEET: &str = "var\\rc2k_hook_plate_b.pcx";

/// Each call: its place (0 lock, 1 blit, 2 unlock) and what it was given: eax, and
/// for the blit ebx, ecx, edx, esi and the piece.
static CALLED: Mutex<Vec<[u32; 10]>> = Mutex::new(Vec::new());
/// Whether the lock in this case succeeds.
static LOCKS: Mutex<bool> = Mutex::new(true);

fn handler(place: u32, r: &mut Regs) {
    // The lock and the unlock read eax alone.
    let noted = if place == 1 {
        let piece = unsafe { (r.ebx as *const [u32; 4]).read_unaligned() };
        [
            place, r.eax, r.ebx, r.ecx, r.edx, r.esi, piece[0], piece[1], piece[2], piece[3],
        ]
    } else {
        [place, r.eax, 0, 0, 0, 0, 0, 0, 0, 0]
    };
    CALLED.lock().unwrap().push(noted);
    if place == 0 {
        r.set_carry(!*LOCKS.lock().unwrap());
    }
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
            &|callee| {
                [LOCK_RVA, BLIT_RVA, UNLOCK_RVA]
                    .iter()
                    .position(|&known| known == callee)
                    .map(|place| stand_in(place as u32))
            },
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
    let inputs: [Region; 6] = [
        (record, RECORD_LEN),
        (game_addr(TEXTURE_RVA) as u32, STORED_LEN),
        at(READING_RVA),
        at(SCALE_RVA),
        at(SATURATION_RVA),
        (game_addr(GRADES_RVA) as u32, 0x100),
    ];
    let saved = capture(&inputs);
    // The pixel kind's description: +4 bytes a pixel, +8 the format.
    let mut description = [0u32; 8];
    // (sheet's size, the kept one's, the texture's, +0x5C, the lock works, graded)
    let cases = [
        (64u16, 64u16, (64u32, 64u32), 0u32, true, false),
        (64, 64, (32, 32), 0, true, false),
        (128, 64, (128, 128), 1, true, false),
        (64, 64, (64, 64), 1, false, false),
        (32, 32, (32, 32), 1, true, true),
        (64, 32, (256, 256), 0, true, true),
    ];
    for (case, &(size, kept, (width, height), keeps, locks, graded)) in cases.iter().enumerate() {
        let made = std::fs::write(
            SHEET,
            crate::gfx::read_pcx_24bit::test_file(size, size, 3, size, case as u32 + 1),
        )
        .and_then(|_| {
            std::fs::write(
                KEPT_SHEET,
                crate::gfx::read_pcx_24bit::test_file(kept, kept, 3, kept, case as u32 + 9),
            )
        });
        if made.is_err() {
            t.check(false, || "cannot write the test's sheets".into());
            continue;
        }
        let format = TEST_FORMATS[case % 3];
        description[1] = 4;
        description[2] = format.layout;
        description[3..7].copy_from_slice(&format.masks);
        let paths = [
            [SHEET.as_bytes(), &[0]].concat(),
            [KEPT_SHEET.as_bytes(), &[0]].concat(),
        ];
        let run = |rust: bool| unsafe {
            loader.prepare();
            let mut fields = [0u8; RECORD_LEN];
            fields[4..8].copy_from_slice(&width.to_le_bytes());
            fields[8..12].copy_from_slice(&height.to_le_bytes());
            fields[0xC..0x10].copy_from_slice(&(description.as_ptr() as u32).to_le_bytes());
            fields[0x5C..0x60].copy_from_slice(&keeps.to_le_bytes());
            std::ptr::copy_nonoverlapping(fields.as_ptr(), record as *mut u8, RECORD_LEN);
            game_write(READING_RVA, graded as u32);
            game_write(SCALE_RVA, 0xC0u32);
            game_write(SATURATION_RVA, 0x140u32);
            game_write(GRADES_RVA, [0xAAu8; 0x100]);
            game_write(TEXTURE_RVA, [0xAAAA_AAAAu32; 7]);
            *LOCKS.lock().unwrap() = locks;
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let regs = Regs {
                eax: paths[0].as_ptr() as u32,
                ebx: TEST_TEXTURE,
                ecx: paths[1].as_ptr() as u32,
                edx: 0x24,
                esi: 0x18,
                edi: 0xAD,
                ebp: 0x68,
                ..Regs::SENTINEL
            };
            let ok = if rust {
                upload_plate(&regs, &mut |call| {
                    let (place, mut given) = match call {
                        Call::Lock(texture) => (
                            0,
                            Regs {
                                eax: texture,
                                ..regs
                            },
                        ),
                        Call::Blit(given) => (1, given),
                        Call::Unlock(texture) => (
                            2,
                            Regs {
                                eax: texture,
                                ..regs
                            },
                        ),
                    };
                    act(place, &mut given);
                    !given.carry()
                })
                .is_ok()
            } else {
                let mut r = regs;
                call_game(&mut r, copy.addr());
                r.preserved_except(&regs, 0)
            };
            (
                ok,
                CALLED.lock().unwrap().clone(),
                capture(&inputs[..2]),
                capture(&inputs[5..]),
                loader.observed(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0, || {
            let first = theirs.4.iter().zip(&mine.4).position(|(a, b)| a != b);
            format!(
                "case {case}: sheet {size}, kept {kept}, texture {width} x {height}, keeps {keeps}, locks {locks}, graded {graded}: ok {} / {}; calls {:x?} / {:x?}; record and stores same {}, grades same {}, first different byte of the picture {first:?}",
                theirs.0, mine.0, theirs.1, mine.1, theirs.2 == mine.2, theirs.3 == mine.3
            )
        });
    }
    let _ = std::fs::remove_file(SHEET);
    let _ = std::fs::remove_file(KEPT_SHEET);
    restore(&inputs, &saved);
    loader.end();
    t.finish()
}
