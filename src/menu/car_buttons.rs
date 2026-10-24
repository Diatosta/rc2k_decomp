//! The variables: `dword_755990` the car shown (an entry of the carousel's records, up
//! to `dword_75649C`), `dword_755994` the slideshow's picture (0 .. 99),
//! `dword_755998` the specification's page (0 .. 3), `dword_755561` the screen loop's
//! leaving switch. Each button is `pusha` .. `popa`.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
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

/// The game's: each call through the Rust version.
pub unsafe fn call_in_game(call: Call) -> Result<(), Refused> {
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
            Call::Stream => play_car_stream(&mut crate::sound::start_stream::start_stream),
            Call::Picture(way) => load_slideshow_picture_in_game(way),
        }
    }
}
