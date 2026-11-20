//! The variables: `event.shown.car` the car shown (an entry of the carousel's records, up to
//! `car_table.last_car`), `event.shown.picture` the slideshow's picture (0 .. 99),
//! `event.shown.car_page` the specification's page (0 .. 3), `loop_flags.leave_screen` the screen
//! loop's leaving switch.

use crate::errors::Refused;
use crate::menu::load_car_background::load_car_background;
use crate::menu::load_slideshow_picture::load_slideshow_picture_in;
use crate::menu::owner::Menus;
use crate::owners::Owners;
use crate::sound::play_car_stream::play_car_stream;

pub use rc2k_core::menu::cars::{Button as Which, Call};

/// A button through `rc2k_core::menu::cars::press`: the variables loaded, changed and
/// stored, then its calls made.
pub fn press(
    menus: &Menus,
    b: Which,
    call: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    use rc2k_core::menu::cars::Shown;

    let mut s = Shown {
        car: menus.screens.event.shown.car.get(),
        last_car: menus.cars.table.last_car.get(),
        picture: menus.screens.event.shown.picture.get(),
        page: menus.screens.event.shown.car_page.get(),
        leave: menus.screens.loop_flags.read().leave_screen,
    };
    let calls = rc2k_core::menu::cars::press(b, &mut s);
    menus.screens.event.shown.car.set(s.car);
    menus.screens.event.shown.picture.set(s.picture);
    menus.screens.event.shown.car_page.set(s.page);
    menus.screens.loop_flags.write().leave_screen = s.leave;
    for &c in calls {
        call(c)?;
    }

    Ok(())
}

/// A button's call made.
pub(super) fn make_call(o: Owners<'static>, call: Call) -> Result<(), Refused> {
    match call {
        Call::Background => load_car_background(o.menus, o.gfx, o.system),
        Call::Stream => play_car_stream(
            &mut |name, volume| {
                crate::sound::start_stream::start_stream(o, name, volume);
            },
            o.menus,
            o.sound,
            &o.audio.sample_names,
        ),
        Call::Picture(way) => load_slideshow_picture_in(o, way),
    }
}
