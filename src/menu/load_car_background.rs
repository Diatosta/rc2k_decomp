//! `menu::load_car_background` (0x468482): the car specification page's background,
//! the car's own.
//!
//! In:  nothing
//! Out: every register preserved; carry as `menu::load_background` leaves it
//!
//! The car is the first dword of the shown entry's record (`dword_755990` through
//! `dword_75A65C`), 0x21 counting as 0x17; the path is that car's in the page's table
//! (`off_759AD2[dword_755998]`); `menu::load_background` with it and ebx = the car (the
//! blit's palette). A carry after it is fatal error 0x80712345. Five call sites, the
//! car specification screens' set-ups.
//! Shadow mode runs the original alone (it loads a picture).
//! Refuses a page past the four, and what the record's lookup refuses.
//! Self-test: a redirected copy whose `load_background` notes what it gets, every page
//! with the game's records.

use crate::errors::{FatalError, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::menu::car_description::{read, shown_description};
use crate::menu::load_background::load_background;
use crate::redirect::RedirectedCopy;
use crate::screens::car_caption::{TEST_CARS, TestRecords};
use crate::service::harness::{at, capture, restore};
use crate::text::draw_text::GameCanvas;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::load_car_background", rva(0x468482), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const PAGE_RVA: usize = rva(0x755998); // dword_755998
const CAR_RVA: usize = rva(0x755990); // dword_755990
const PATHS_RVA: usize = rva(0x759AD2); // off_759AD2: a table of paths a page
const LOAD_RVA: usize = rva(0x47EFFF); // menu::load_background

// ---- Rust version ----

/// The car of the entry shown, 0x21 counting as 0x17.
pub fn shown_car() -> Result<u32, Refused> {
    let car = unsafe { read(shown_description()?) }.car;
    // Converted: `rc2k_core::menu::slides::picture_car`.
    Ok(rc2k_core::menu::slides::picture_car(car))
}

/// `load` gets the path and the car; it returns whether the carry was set.
pub unsafe fn load_car_background(
    load: &mut dyn FnMut(u32, u32) -> Result<bool, Refused>,
) -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::menu::slides::background_page`.
        let page = rc2k_core::menu::slides::background_page(game_read(PAGE_RVA))?;
        let car = shown_car()?;
        let table: u32 = game_read(PATHS_RVA + page as usize * 4);
        let path = ((table + car * 4) as *const u32).read_unaligned();
        if load(path, car)? {
            return Err(FatalError {
                code: 0x8071_2345,
                message: "",
            }
            .into());
        }
        Ok(())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let done = unsafe {
        load_car_background(&mut |path, car| {
            load_background(path as *mut u8, car, &mut GameCanvas)?;
            Ok(false)
        })
    };
    match done {
        Ok(()) => r.set_carry(false),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

const LEN: usize = 0x3B;
const ENTRY_BYTES: [u8; 7] = [0x60, 0x8B, 0x1D, 0x98, 0x59, 0x75, 0x00];
const CALLS: [usize; 1] = [0x4684B0 - 0x468482];

static CALLED: Mutex<Vec<[u32; 2]>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push([r.eax, r.ebx]);
    r.set_carry(false);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x468482)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == LOAD_RVA).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let inputs = [at(PAGE_RVA), at(CAR_RVA)];
    let saved = capture(&inputs);
    set_handler(handler);
    let records = TestRecords::new(&TEST_CARS, &[0x249]);
    let entries: Vec<u32> = (0..records.len()).collect();
    for page in 0..rc2k_core::menu::slides::PAGES {
        for &car in &entries {
            let run = |rust: bool| unsafe {
                game_write(PAGE_RVA, page);
                game_write(CAR_RVA, car);
                CALLED.lock().unwrap().clear();
                let before = Regs::SENTINEL;
                let mut r = before;
                let kept = if rust {
                    load_car_background(&mut |path, car| {
                        act(
                            0,
                            &mut Regs {
                                eax: path,
                                ebx: car,
                                ..before
                            },
                        );
                        Ok(false)
                    })
                    .is_ok()
                } else {
                    call_game(&mut r, copy.addr());
                    r.preserved_except(&before, 0)
                };
                (kept, CALLED.lock().unwrap().clone())
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && theirs.0, || {
                format!("page {page}, entry {car}: original {theirs:x?}; rust {mine:x?}")
            });
        }
    }
    drop(records);
    restore(&inputs, &saved);
    t.finish()
}
