//! `menu::load_slideshow_picture` (0x4684F5): the slideshow's picture of the car shown,
//! the next one that is there.
//!
//! In:  eax = the way to look (below 0: back, else on)
//! Out: every register preserved
//!
//! The car (as `menu::load_car_background` finds it, 0x21 counting as 0x17); its
//! pictures are `off_759B0E[car]` (`;3files\g\slides\mgnslNN.png` and so on) with the
//! number `dword_755994`. Starting from that number: the path made
//! (`files::numbered_path`); if the file is there it is taken, else the path's last two
//! characters become "cx" (`.pcx`) and if that is there it is taken; else the number
//! steps the way (from 0 back to 99, from 99 on to 0) and the next is tried, until the
//! number comes round to where it started, when the last path tried is taken anyway.
//! Then `gfx::load_picture_with_overlay` with it and the title bar
//! (`files\g\graphics\screens\titlebar.png`), or, if that could not read it,
//! `gfx::load_picture` of the title bar alone; then `gfx::convert_picture`.
//! Five call sites: the slideshow's buttons and its set-up.
//! Shadow mode runs the original alone.
//! Refuses a car with no pictures (only the first 0x18 have them), and what its parts
//! refuse.
//! Self-test: a redirected copy whose loading and converting note what they get, on
//! the owner's own files, nine cars both ways from three numbers.

use crate::errors::{BadInput, Refused};
use crate::files::file_exists::file_exists;
use crate::files::numbered_path::numbered_path_in_game;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_ptr, game_read, game_write, hook_asm, rva,
    shadow,
};
use crate::gfx::convert_picture::convert_picture_now;
use crate::gfx::load_picture::load_picture;
use crate::gfx::load_picture_with_overlay::load_picture_with_overlay;
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::menu::car_buttons::{CAR_RVA, PICTURE_RVA};
use crate::menu::load_car_background::shown_car;
use crate::redirect::RedirectedCopy;
use crate::screens::car_caption::TestRecords;
use crate::service::harness::{at, capture, restore};
use crate::strings::text_length::text_length;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::load_slideshow_picture",
    rva(0x4684F5),
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

const TEMPLATES_RVA: usize = rva(0x759B0E); // off_759B0E
const TITLE_BAR_RVA: usize = rva(0x731C54); // "files\g\graphics\screens\titlebar.png"
const CARS: u32 = 0x18;
const PICTURES: i32 = 100;

/// What it asks for.
pub enum Step {
    /// `gfx::load_picture_with_overlay` with the path and the title bar: the carry.
    Overlay(*mut u8, *mut u8),
    /// `gfx::load_picture` of the title bar.
    Load(*mut u8),
    Convert,
}

// ---- Rust version ----

pub unsafe fn load_slideshow_picture(
    way: u32,
    step: &mut dyn FnMut(Step) -> Result<bool, Refused>,
) -> Result<(), Refused> {
    unsafe {
        let car = shown_car()?;
        if car >= CARS {
            return Err(BadInput("a car with no slideshow").into());
        }
        let template: u32 = game_read(TEMPLATES_RVA + car as usize * 4);
        let start: u32 = game_read(PICTURE_RVA);
        let path = loop {
            let number: u32 = game_read(PICTURE_RVA);
            let path = numbered_path_in_game(template as *const u8, number)? as *mut u8;
            if file_exists(path)? {
                break path;
            }
            let length = text_length(path)? as usize;
            if length < 2 {
                return Err(BadInput("a picture's path of fewer than two characters").into());
            }
            *path.add(length - 2) = b'c';
            *path.add(length - 1) = b'x';
            if file_exists(path)? {
                break path;
            }
            let next = if (way as i32) < 0 {
                let n = (number as i32).wrapping_sub(1);
                if n < 0 { PICTURES - 1 } else { n }
            } else {
                let n = (number as i32).wrapping_add(1);
                if n > PICTURES - 1 { 0 } else { n }
            };
            game_write(PICTURE_RVA, next);
            if next as u32 == start {
                break path;
            }
        };
        let title_bar: *mut u8 = game_ptr(TITLE_BAR_RVA);
        if step(Step::Overlay(path, title_bar))? {
            step(Step::Load(title_bar))?;
        }
        step(Step::Convert)?;
        Ok(())
    }
}

/// With the Rust versions of what it calls.
pub unsafe fn load_slideshow_picture_in_game(way: u32) -> Result<(), Refused> {
    unsafe {
        load_slideshow_picture(way, &mut |step| match step {
            Step::Overlay(path, over) => load_picture_with_overlay(path, over),
            Step::Load(path) => load_picture(path).map(|_| false),
            Step::Convert => convert_picture_now().map(|_| false).map_err(Into::into),
        })
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { load_slideshow_picture_in_game(r.eax) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0xAD;
const ENTRY_BYTES: [u8; 6] = [0x60, 0x8B, 0xE8, 0x8B, 0x1D, 0x90];
const CALLS: [usize; 7] = [0x31, 0x36, 0x3F, 0x4B, 0x98, 0xA1, 0xA6];
const STEPS: [usize; 3] = [rva(0x4118D1), rva(0x411863), rva(0x406126)];

/// Each step: its place, and the path it got (up to 0x40 bytes) and ebx.
static CALLED: Mutex<Vec<(u32, Vec<u8>, u32)>> = Mutex::new(Vec::new());
static OVERLAY_FAILS: AtomicBool = AtomicBool::new(false);

fn handler(place: u32, r: &mut Regs) {
    // Only the two loadings have a path in eax; the conversion's eax is a leftover.
    let path = if place == 2 || r.eax == 0 {
        Vec::new()
    } else {
        unsafe { std::ffi::CStr::from_ptr(r.eax as *const std::ffi::c_char) }
            .to_bytes()
            .iter()
            .take(0x40)
            .copied()
            .collect()
    };
    CALLED.lock().unwrap().push((place, path, r.ebx));
    if place == 0 {
        r.set_carry(OVERLAY_FAILS.load(Ordering::SeqCst));
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x4684F5)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                STEPS
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
    let inputs = [at(CAR_RVA), at(PICTURE_RVA)];
    let saved = capture(&inputs);
    set_handler(handler);
    let cars = [0u32, 1, 2, 5, 0x10, 0x16, 0x17, 0x21, 3];
    let records = TestRecords::new(&cars, &[0x249]);
    for entry in 0..records.len() {
        for start in [0u32, 7, 99] {
            for (way, fails) in [(1u32, false), (u32::MAX, false), (1, true)] {
                let run = |rust: bool| unsafe {
                    game_write(CAR_RVA, entry);
                    game_write(PICTURE_RVA, start);
                    OVERLAY_FAILS.store(fails, Ordering::SeqCst);
                    CALLED.lock().unwrap().clear();
                    let before = Regs {
                        eax: way,
                        ..Regs::SENTINEL
                    };
                    let kept = if rust {
                        load_slideshow_picture(way, &mut |step| {
                            let (place, eax, ebx) = match step {
                                Step::Overlay(path, over) => (0, path as u32, over as u32),
                                Step::Load(path) => (1, path as u32, path as u32),
                                Step::Convert => (2, 0, 0),
                            };
                            let mut r = Regs { eax, ebx, ..before };
                            act(place, &mut r);
                            Ok(r.carry())
                        })
                        .is_ok()
                    } else {
                        let mut r = before;
                        call_game(&mut r, copy.addr());
                        r.preserved_except(&before, 0)
                    };
                    let noted: Vec<(u32, Vec<u8>, u32)> = CALLED
                        .lock()
                        .unwrap()
                        .iter()
                        .map(|(place, path, ebx)| {
                            // The convert's registers are the loop's leftovers: not read.
                            if *place == 2 {
                                (*place, Vec::new(), 0)
                            } else {
                                (*place, path.clone(), *ebx)
                            }
                        })
                        .collect();
                    (kept, game_read::<u32>(PICTURE_RVA), noted)
                };
                let (theirs, mine) = (run(false), run(true));
                t.check(theirs == mine && theirs.0, || {
                    format!(
                        "car {:#x}, from {start}, way {way:#x}, overlay fails {fails}: original {theirs:x?}; rust {mine:x?}",
                        cars[entry as usize]
                    )
                });
            }
        }
    }
    drop(records);
    restore(&inputs, &saved);
    t.finish()
}
