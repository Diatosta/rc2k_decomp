//! `gfx::upload_texture_picture` (0x41A68B): a picture into every level of a system
//! texture.
//!
//! In:  eax = the system texture, ebx = the picture's pixels (0: nothing is done),
//!      ecx, edx = its width and height
//! Out: carry clear; every register preserved
//!
//! - The picture made ready (`gfx/texture_upload.rs`: the least colour, the alpha from
//!   the kept picture or the colours keyed, the grading).
//! - Picture and texture the same number of halvings from 256 across and down, else a
//!   fatal error (0x80A18741 .. 0x80A18747).
//! - For each of the record's levels (+0x10; one at least): the picture halved (that
//!   many times for the first, once for each after), the level's surface (+0x14 ..)
//!   locked through its `Lock` (DirectDraw, vtable +0x64, into the description at
//!   `dword_5ABA18`), every pixel through `components_to_pixel` with the texture's
//!   format into it as words or dwords by the kind's bytes a pixel (another number: a
//!   fatal error 0x80A18775), rows `dword_5ABA28` apart from `dword_5ABA3C`, and
//!   `Unlock` (+0x80).
//! - A `Lock` or `Unlock` that fails is a fatal error (0x80218772, 0x80218773) when
//!   `dword_4F52AC` is 1; otherwise the original returns with three dwords still
//!   pushed, so its `popa` and `ret` go astray: refused.
//!
//! One caller, `gfx::load_picture_to_texture` (the other reference, 0x425A52, is in code
//! nothing reaches).
//! The two DirectDraw calls go through `Surfaces`, which the standalone program can
//! answer with textures of its own.
//! Refuses a texture of 0x140 or more, a record without its format, more than nine
//! levels, a picture that halves to nothing, and what `ready_picture` refuses; a
//! refusal after the picture was changed hands the original the changed picture.
//! Shadow mode runs the original alone.
//! Self-test: the original and the Rust version on records of the test's whose
//! surfaces are objects of the test's, 12 cases.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::pixel_format::{SCRATCH_RVA, TEST_FORMATS};
use crate::gfx::texture_pixels::{
    GRADES_RVA, KEPT, READING_RVA, SATURATION_RVA, SCALE_RVA, halvable, halve_texture, test_pixels,
};
use crate::gfx::texture_upload::{
    RECORD_LEN, Record, SYSTEM_RECORDS_RVA, TEXTURES, fatal, halvings_apart, ready_picture,
};
use crate::service::harness::{Region, at, capture, restore};
use rc2k_core::gfx::uploads::{self, MOST_LEVELS as LEVELS};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "gfx::upload_texture_picture",
    rva(0x41A68B),
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

// ---- game data ----

/// `dword_5ABA18`: the surface description `Lock` fills (0x6C bytes).
const DESCRIPTION_RVA: usize = rva(0x5ABA18);
const DESCRIPTION_LEN: usize = 0x6C;
const PITCH_RVA: usize = rva(0x5ABA28); // dword_5ABA28: its pitch
const PIXELS_RVA: usize = rva(0x5ABA3C); // dword_5ABA3C: its pixels
const REPORTS_RVA: usize = rva(0x4F52AC); // dword_4F52AC: Direct3D's errors are fatal

// ---- Rust version ----

/// A level's surface locked and unlocked; an error is what DirectDraw returned.
pub trait Surfaces {
    /// The pixels and the pitch.
    fn lock(&mut self, surface: u32) -> Result<(u32, u32), i32>;
    fn unlock(&mut self, surface: u32) -> Result<(), i32>;
}

/// The game's: DirectDraw's own calls, the description being the game's.
pub struct GameSurfaces;

type Lock = unsafe extern "system" fn(u32, u32, u32, u32, u32) -> i32;
type Unlock = unsafe extern "system" fn(u32, u32) -> i32;

unsafe fn method(object: u32, offset: u32) -> usize {
    unsafe {
        (((object as *const u32).read_unaligned() + offset) as *const u32).read_unaligned() as usize
    }
}

impl Surfaces for GameSurfaces {
    fn lock(&mut self, surface: u32) -> Result<(u32, u32), i32> {
        unsafe {
            let lock: Lock = std::mem::transmute(method(surface, 0x64));
            match lock(surface, 0, game_addr(DESCRIPTION_RVA) as u32, 1, 0) {
                0 => Ok((game_read(PIXELS_RVA), game_read(PITCH_RVA))),
                e => Err(e),
            }
        }
    }

    fn unlock(&mut self, surface: u32) -> Result<(), i32> {
        unsafe {
            let unlock: Unlock = std::mem::transmute(method(surface, 0x80));
            match unlock(surface, 0) {
                0 => Ok(()),
                e => Err(e),
            }
        }
    }
}

fn direct_draw_failed(code: u32) -> Refused {
    if unsafe { game_read::<u32>(REPORTS_RVA) } == 1 {
        fatal(code)
    } else {
        BadInput("DirectDraw failed and the original's stack goes astray").into()
    }
}

pub unsafe fn upload_texture_picture(
    texture: u32,
    pixels: u32,
    width: u32,
    height: u32,
    surfaces: &mut dyn Surfaces,
) -> Result<(), Refused> {
    if pixels == 0 {
        return Ok(());
    }
    // Converted: `rc2k_core::gfx::uploads` (`levels`, `halvings_before`; the halvings
    // through `texture_upload::halvings_apart`).
    let record = Record::of(texture)?;
    let levels = uploads::levels(record.get().levels).map_err(BadInput::from)?;
    unsafe {
        ready_picture(&record, pixels, width, height)?;
        let apart = halvings_apart(
            &record,
            width,
            height,
            [
                0x80A1_8741,
                0x80A1_8742,
                0x80A1_8744,
                0x80A1_8745,
                0x80A1_8747,
            ],
        )?;
        if !halvable(width, height, apart + levels - 1) {
            return Err(BadInput("a picture that halves to nothing").into());
        }
        let format = record.format();
        let bytes = record.bytes_per_pixel();
        let (mut width, mut height) = (width, height);
        for level in 0..levels {
            let times = uploads::halvings_before(level, apart);
            if times != 0 {
                (width, height) = halve_texture(pixels as *mut u32, width, height, times);
            }
            let surface = record.get().surfaces[level as usize];
            let (mut target, pitch) = surfaces
                .lock(surface)
                .map_err(|_| direct_draw_failed(0x8021_8772))?;
            let mut source = pixels as *const u32;
            for _ in 0..height {
                match bytes {
                    2 => {
                        for n in 0..width {
                            let pixel = components_to_pixel(&format, source.read_unaligned());
                            ((target + n * 2) as *mut u16).write_unaligned(pixel as u16);
                            source = source.add(1);
                        }
                    }
                    4 => {
                        for n in 0..width {
                            let pixel = components_to_pixel(&format, source.read_unaligned());
                            ((target + n * 4) as *mut u32).write_unaligned(pixel);
                            source = source.add(1);
                        }
                    }
                    _ => return Err(fatal(0x80A1_8775)),
                }
                target = target.wrapping_add(pitch);
            }
            surfaces
                .unlock(surface)
                .map_err(|_| direct_draw_failed(0x8021_8773))?;
        }
    }
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if crate::framework::shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { upload_texture_picture(r.eax, r.ebx, r.ecx, r.edx, &mut GameSurfaces) } {
        Ok(()) => r.set_carry(false),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    match unsafe { upload_texture_picture(r.eax, r.ebx, r.ecx, r.edx, &mut GameSurfaces) } {
        Ok(()) => r.set_carry(false),
        Err(e) => return Err(e),
    }
    Ok(())
}

// ---- self-test ----

const TEST_TEXTURE: u32 = TEXTURES - 1;

/// The test's surfaces: each a pointer to `VTABLE` and its level's number. Each lock is
/// noted, with the level, and points the description at the level's pixels.
struct TestSurfaces {
    levels: Vec<Vec<u8>>,
    pitch: u32,
}

static TEST: Mutex<Option<TestSurfaces>> = Mutex::new(None);
static CALLED: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());

unsafe extern "system" fn test_lock(
    this: u32,
    _: u32,
    description: u32,
    flags: u32,
    _: u32,
) -> i32 {
    let level = unsafe { ((this + 4) as *const u32).read_unaligned() };
    CALLED.lock().unwrap().push((0, level | flags << 8));
    let mut test = TEST.lock().unwrap();
    let test = test.as_mut().unwrap();
    unsafe {
        ((description + 0x24) as *mut u32)
            .write_unaligned(test.levels[level as usize].as_mut_ptr() as u32);
        ((description + 0x10) as *mut u32).write_unaligned(test.pitch);
    }
    0
}

unsafe extern "system" fn test_unlock(this: u32, _: u32) -> i32 {
    let level = unsafe { ((this + 4) as *const u32).read_unaligned() };
    CALLED.lock().unwrap().push((1, level));
    0
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let original = ORIGINAL.load(Ordering::SeqCst);
    let record = game_addr(SYSTEM_RECORDS_RVA) as u32 + TEST_TEXTURE * RECORD_LEN as u32;
    let inputs: [Region; 8] = [
        (record, RECORD_LEN),
        (game_addr(DESCRIPTION_RVA) as u32, DESCRIPTION_LEN),
        at(READING_RVA),
        at(SCALE_RVA),
        at(SATURATION_RVA),
        (game_addr(GRADES_RVA) as u32, 0x100),
        at(SCRATCH_RVA),
        at(REPORTS_RVA),
    ];
    let saved = capture(&inputs);
    let mut vtable = [0u32; 0x21];
    vtable[0x64 / 4] = test_lock as *const () as u32;
    vtable[0x80 / 4] = test_unlock as *const () as u32;
    let objects: Vec<[u32; 2]> = (0..LEVELS).map(|n| [vtable.as_ptr() as u32, n]).collect();
    // (picture, texture, levels, bytes a pixel, +0x5C, graded)
    let cases = [
        (32u32, 32u32, 1u32, 4u32, 0u32, false),
        (32, 32, 3, 2, 0, false),
        (64, 32, 1, 4, 0, false),
        (128, 32, 2, 2, 1, false),
        (64, 64, 4, 4, 1, false),
        (32, 64, 1, 4, 0, false),
        (32, 32, 0, 2, 0, true),
        (64, 64, 5, 4, 0, true),
        (128, 128, 2, 2, 1, true),
        (256, 128, 3, 4, 0, false),
        (64, 32, 2, 4, 1, true),
        (32, 32, 1, 2, 1, false),
    ];
    let mut seed = 0x41A6_8B00u32;
    for (case, &(size, texture, levels, bytes, keeps, graded)) in cases.iter().enumerate() {
        let n = (size * size) as usize;
        let picture = test_pixels(&mut seed, n);
        let kept = test_pixels(&mut seed, n);
        let format = TEST_FORMATS[[1, 2, 0, 4][case % 4]];
        let description: [u32; 8] = [
            0,
            bytes,
            format.layout,
            format.masks[0],
            format.masks[1],
            format.masks[2],
            format.masks[3],
            0,
        ];
        let pitch = texture * bytes + 8;
        let run = |rust: bool| unsafe {
            let mut buffer = vec![0xAAAA_AAAAu32; KEPT / 4 + n];
            buffer[..n].copy_from_slice(&picture);
            buffer[KEPT / 4..].copy_from_slice(&kept);
            *TEST.lock().unwrap() = Some(TestSurfaces {
                levels: vec![vec![0xAAu8; (pitch * texture) as usize]; LEVELS as usize],
                pitch,
            });
            CALLED.lock().unwrap().clear();
            let mut fields = [0u32; RECORD_LEN / 4];
            fields[1] = texture;
            fields[2] = texture;
            fields[3] = description.as_ptr() as u32;
            fields[4] = levels;
            for (level, object) in objects.iter().enumerate() {
                fields[5 + level] = object.as_ptr() as u32;
            }
            fields[0x5C / 4] = keeps;
            std::ptr::copy_nonoverlapping(
                fields.as_ptr() as *const u8,
                record as *mut u8,
                RECORD_LEN,
            );
            crate::framework::game_write(READING_RVA, graded as u32);
            crate::framework::game_write(SCALE_RVA, 0xA0u32);
            crate::framework::game_write(SATURATION_RVA, 0x120u32);
            crate::framework::game_write(GRADES_RVA, [0xAAu8; 0x100]);
            let ok = if rust {
                upload_texture_picture(
                    TEST_TEXTURE,
                    buffer.as_ptr() as u32,
                    size,
                    size,
                    &mut GameSurfaces,
                )
                .is_ok()
            } else {
                let before = Regs {
                    eax: TEST_TEXTURE,
                    ebx: buffer.as_ptr() as u32,
                    ecx: size,
                    edx: size,
                    ..Regs::SENTINEL
                };
                let mut r = before;
                call_game(&mut r, original);
                r.preserved_except(&before, 0) && !r.carry()
            };
            let levels = TEST.lock().unwrap().take().unwrap().levels;
            // Where the test's lock put each run's own pixels: not compared.
            crate::framework::game_write(DESCRIPTION_RVA + 0x24, 0u32);
            (
                ok,
                CALLED.lock().unwrap().clone(),
                levels,
                buffer[..n].to_vec(),
                capture(&inputs[..7]),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0, || {
            let level = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "case {case}: picture {size}, texture {texture}, {levels} levels, {bytes} bytes, keeps {keeps}, graded {graded}: ok {} / {}; calls {:x?} / {:x?}; first different level {level:?}; picture same {}; record, description and the rest same {}",
                theirs.0, mine.0, theirs.1, mine.1, theirs.3 == mine.3, theirs.4 == mine.4
            )
        });
    }
    restore(&inputs, &saved);
    t.finish()
}
