//! The variables: `dword_755990` the car shown (an entry of the carousel's records, up
//! to `dword_75649C`), `dword_755994` the slideshow's picture (0 .. 99),
//! `dword_755998` the specification's page (0 .. 3), `dword_755561` the screen loop's
//! leaving switch. Each button is `pusha` .. `popa`.

use crate::errors::Refused;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::load_car_background::load_car_background;
use crate::menu::load_slideshow_picture::load_slideshow_picture_in_game;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::sound::play_car_stream::play_car_stream;
use crate::text::draw_text::GameCanvas;

pub use rc2k_core::menu::cars::{Button as Which, Call};

/// A button through `rc2k_core::menu::cars::press`: the variables loaded, changed and
/// stored, then its calls made.
pub fn press(b: Which, call: &mut dyn FnMut(Call) -> Result<(), Refused>) -> Result<(), Refused> {
    use rc2k_core::menu::cars::Shown;

    let mut s = Shown {
        car: EVENT_STATE.shown_car.get(),
        last_car: crate::menu::owner::menus().cars.table.last_car.get(),
        picture: EVENT_STATE.picture.get(),
        page: EVENT_STATE.car_page.get(),
        leave: LOOP_FLAGS.back.get(),
    };
    let calls = rc2k_core::menu::cars::press(b, &mut s);
    EVENT_STATE.shown_car.set(s.car);
    EVENT_STATE.picture.set(s.picture);
    EVENT_STATE.car_page.set(s.page);
    LOOP_FLAGS.back.set(s.leave);
    for &c in calls {
        call(c)?;
    }

    Ok(())
}

/// The game's: each call through the Rust version.
pub fn call_in_game(call: Call) -> Result<(), Refused> {
    match call {
        Call::Background => load_car_background(&mut |path| {
            crate::menu::load_background::load_background(&path, &mut GameCanvas)?;
            Ok(false)
        }),
        Call::Stream => play_car_stream(&mut crate::sound::start_stream::start_stream),
        Call::Picture(way) => load_slideshow_picture_in_game(way),
    }
}
