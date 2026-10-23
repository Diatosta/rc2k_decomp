//! What the car specification and slideshow screens' buttons share (0x469AFD,
//! 0x469BD1, 0x469B35, 0x469B70, 0x469BF4, 0x469C1A): the variables they change, the
//! three things they call, and one self-test for all of them.
//!
//! The variables: `dword_755990` the car shown (an entry of the carousel's records, up
//! to `dword_75649C`), `dword_755994` the slideshow's picture (0 .. 99),
//! `dword_755998` the specification's page (0 .. 3), `dword_755561` the screen loop's
//! leaving switch. Each button is `pusha` .. `popa`.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::menu::load_car_background::load_car_background;
use crate::menu::load_slideshow_picture::load_slideshow_picture_in_game;
use crate::sound::play_car_stream::play_car_stream;
use crate::text::draw_text::GameCanvas;

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
            Call::Stream => play_car_stream(r, &mut |r| crate::calls::call(CALLEES[1], r)),
            Call::Picture(way) => load_slideshow_picture_in_game(way),
        }
    }
}
