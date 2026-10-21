//! What the car specification and slideshow screens' buttons share (0x469AFD,
//! 0x469BD1, 0x469B35, 0x469B70, 0x469BF4, 0x469C1A): the variables they change, the
//! three things they call, and one self-test for all of them.
//!
//! The variables: `dword_755990` the car shown (an entry of the carousel's records, up
//! to `dword_75649C`), `dword_755994` the slideshow's picture (0 .. 99),
//! `dword_755998` the specification's page (0 .. 3), `dword_755561` the screen loop's
//! leaving switch. Each button is `pusha` .. `popa`.

use crate::errors::Refused;
use crate::framework::{Regs, SelfTest, call_game, game_addr, game_read, game_write, rva};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::menu::load_car_background::load_car_background;
use crate::menu::load_slideshow_picture::load_slideshow_picture_in_game;
use crate::redirect::RedirectedCopy;
use crate::screens::car_caption::TestRecords;
use crate::service::harness::{Region, at, capture, restore};
use crate::sound::play_car_stream::play_car_stream;
use crate::text::draw_text::GameCanvas;
use std::sync::Mutex;

pub const CAR_RVA: usize = rva(0x755990); // dword_755990
pub const PICTURE_RVA: usize = rva(0x755994); // dword_755994
pub const PAGE_RVA: usize = rva(0x755998); // dword_755998
pub const LEAVE_RVA: usize = rva(0x755561); // dword_755561
pub const LAST_CAR_RVA: usize = rva(0x75649C); // dword_75649C

pub use rc2k_core::menu::cars::{Button as Which, Call};

/// A button through `rc2k_core::menu::cars::press`: the variables loaded, changed and
/// stored, then its calls made.
pub unsafe fn press(
    b: Which,
    call: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    use rc2k_core::menu::cars::Shown;
    unsafe {
        let mut s = Shown {
            car: game_read(CAR_RVA),
            last_car: game_read(LAST_CAR_RVA),
            picture: game_read(PICTURE_RVA),
            page: game_read(PAGE_RVA),
            leave: game_read(LEAVE_RVA),
        };
        let calls = rc2k_core::menu::cars::press(b, &mut s);
        game_write(CAR_RVA, s.car);
        game_write(PICTURE_RVA, s.picture);
        game_write(PAGE_RVA, s.page);
        game_write(LEAVE_RVA, s.leave);
        for &c in calls {
            call(c)?;
        }
    }
    Ok(())
}

const CALLEES: [usize; 3] = [rva(0x468482), rva(0x469AA1), rva(0x4684F5)];

/// The game's: each call through the Rust version.
pub unsafe fn call_in_game(call: Call, r: &Regs) -> Result<(), Refused> {
    unsafe {
        match call {
            Call::Background => load_car_background(&mut |path, car| {
                crate::menu::load_background::load_background(
                    path as *mut u8,
                    car,
                    &mut GameCanvas,
                )?;
                Ok(false)
            }),
            Call::Stream => play_car_stream(r, &mut |r| call_game(r, game_addr(CALLEES[1]))),
            Call::Picture(way) => load_slideshow_picture_in_game(way),
        }
    }
}

/// A button's Rust version, its calls going to the function it is given.
pub type Button = dyn Fn(&mut dyn FnMut(Call) -> Result<(), Refused>) -> Result<(), Refused>;

/// A button's original, where it lies.
pub struct Original {
    pub address: usize,
    pub len: usize,
    pub entry: [u8; 6],
    pub calls: &'static [usize],
}

/// Each call: its place, eax, and the four variables by then.
static CALLED: Mutex<Vec<[u32; 6]>> = Mutex::new(Vec::new());

fn variables() -> [Region; 5] {
    [
        at(CAR_RVA),
        at(PICTURE_RVA),
        at(PAGE_RVA),
        at(LEAVE_RVA),
        at(LAST_CAR_RVA),
    ]
}

fn handler(place: u32, r: &mut Regs) {
    unsafe {
        CALLED.lock().unwrap().push([
            place,
            // Only the picture's loading reads eax (the way); for the other two it is a
            // leftover.
            if place == 2 { r.eax } else { 0 },
            game_read(CAR_RVA),
            game_read(PICTURE_RVA),
            game_read(PAGE_RVA),
            game_read(LEAVE_RVA),
        ]);
    }
}

/// The original's copy against the Rust version (`rust`, its calls going to `step`), over
/// cars, pictures and pages at their ends and between.
pub fn check_button(t: &mut SelfTest, original: &Original, rust: &Button) {
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(original.address)),
            original.len,
            &original.entry,
            original.calls,
            &|callee| {
                CALLEES
                    .iter()
                    .position(|&known| known == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => {
            t.check(false, || format!("no copy: {why}"));
            return;
        }
    };
    let saved = capture(&variables());
    let records = TestRecords::new(&[0, 1, 2, 3, 0x21], &[0x249]);
    set_handler(handler);
    for last in [0u32, 4, 0xE] {
        for car in [0u32, 1, last, last.wrapping_add(1), 0xFFFF_FFFF] {
            for (picture, page) in [(0u32, 0u32), (50, 1), (99, 3), (100, 2), (0xFFFF_FFFF, 4)] {
                let run = |rust_side: bool| unsafe {
                    game_write(CAR_RVA, car);
                    game_write(PICTURE_RVA, picture);
                    game_write(PAGE_RVA, page);
                    game_write(LEAVE_RVA, 7u32);
                    game_write(LAST_CAR_RVA, last);
                    CALLED.lock().unwrap().clear();
                    let before = Regs::SENTINEL;
                    let kept = if rust_side {
                        rust(&mut |call| {
                            let (place, eax) = match call {
                                Call::Background => (0, before.eax),
                                Call::Stream => (1, before.eax),
                                Call::Picture(way) => (2, way),
                            };
                            act(place, &mut Regs { eax, ..before });
                            Ok(())
                        })
                        .is_ok()
                    } else {
                        let mut r = before;
                        call_game(&mut r, copy.addr());
                        r.preserved_except(&before, 0)
                    };
                    (kept, capture(&variables()), CALLED.lock().unwrap().clone())
                };
                let (theirs, mine) = (run(false), run(true));
                t.check(theirs == mine && theirs.0, || {
                    format!(
                        "last car {last}, car {car:#x}, picture {picture:#x}, page {page}: original {theirs:x?}; rust {mine:x?}"
                    )
                });
            }
        }
    }
    drop(records);
    restore(&variables(), &saved);
}
