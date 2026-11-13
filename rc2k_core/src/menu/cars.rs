//! The car specification and slideshow screens' buttons: what each changes, then what it calls;
//! the car choice's carousel (`menu::find_player_car`, `next_choice_car`, `previous_choice_car`);
//! and the car specifications' list by the kind of event (`menu::show_car_specs`).

/// What the screens show: the car (an entry up to `last_car`), the slideshow's picture
/// (0 .. 99), the specification's page (0 .. 3), the screen loop's leaving switch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Shown {
    pub car: u32,
    pub last_car: u32,
    pub picture: u32,
    pub page: u32,
    pub leave: bool,
}

/// What a button calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    /// The car's background.
    Background,
    /// The car's speech.
    Stream,
    /// The slideshow's picture, the way given (1 on, -1 back).
    Picture(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    NextCar,
    /// `menu::previous_car`: from 0 to the last.
    PreviousCar,
    NextPage,
    /// `menu::previous_car_page`: from 0 and below (signed) to 3.
    PreviousPage,
    PreviousSlideshowCar,
    NextSlideshowCar,
    PreviousPicture,
    NextPicture,
}

/// A button pressed: `s` changed, and the calls to make after, in order.
pub fn press(b: Button, s: &mut Shown) -> &'static [Call] {
    match b {
        Button::NextCar => {
            s.car = if s.car == s.last_car {
                0
            } else {
                s.car.wrapping_add(1)
            };
            s.leave = false;
            &[Call::Background, Call::Stream]
        }
        Button::PreviousCar => {
            s.car = if s.car == 0 {
                s.last_car
            } else {
                s.car.wrapping_sub(1)
            };
            s.leave = false;
            &[Call::Background, Call::Stream]
        }
        Button::NextPage => {
            s.page = if s.page == 3 {
                0
            } else {
                s.page.wrapping_add(1)
            };
            &[Call::Background]
        }
        Button::PreviousPage => {
            s.page = if (s.page as i32) > 0 { s.page - 1 } else { 3 };
            &[Call::Background]
        }
        Button::PreviousSlideshowCar | Button::NextSlideshowCar => {
            let car = if b == Button::NextSlideshowCar {
                let car = s.car.wrapping_add(1);
                if car as i32 > s.last_car as i32 {
                    0
                } else {
                    car
                }
            } else {
                let car = s.car.wrapping_sub(1);
                if (car as i32) < 0 { s.last_car } else { car }
            };
            (s.car, s.leave, s.picture) = (car, false, 0);
            &[Call::Picture(1), Call::Stream]
        }
        Button::PreviousPicture => {
            let p = s.picture.wrapping_sub(1);
            s.picture = if (p as i32) < 0 { 99 } else { p };
            &[Call::Picture(u32::MAX)]
        }
        Button::NextPicture => {
            let p = s.picture.wrapping_add(1);
            s.picture = if p as i32 > 99 { 0 } else { p };
            &[Call::Picture(1)]
        }
    }
}

/// The car lists the car specifications screen shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecsList {
    /// `MenuLists.cars_second[0]` (kinds 1 .. 3).
    Events,
    /// `MenuLists.cars_championship[0]` (kind 4).
    Kind4,
    /// `MenuLists.a8_championship_cars[0]` (kind 5).
    Kind5,
}

/// `menu::show_car_specs`: the list for the kind of event, and whether the screen is a choice (the
/// cheat's list may stand in, the car shown is taken unless the screen was left) or shown alone
/// (kind 5). None: another kind, a fatal error, "No GameMode Defined".
pub fn specs_list(kind: u32) -> Option<(SpecsList, bool)> {
    match kind {
        1..=3 => Some((SpecsList::Events, true)),
        4 => Some((SpecsList::Kind4, true)),
        5 => Some((SpecsList::Kind5, false)),
        _ => None,
    }
}

/// The car choice's carousel: its turn (the front, the direction and the angle turn as
/// `navigation::turn` turns its counter) and the shown entry, up to `last`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Carousel {
    pub turn: crate::menu::navigation::Turn,
    pub shown: u32,
    pub last: u32,
}

/// The carousel turned to the next car (`on`) or the one before; the shown entry round.
pub fn turn_carousel(c: Carousel, on: bool) -> Carousel {
    Carousel {
        turn: crate::menu::navigation::turn(c.turn, on),
        shown: if on {
            if c.shown == c.last {
                0
            } else {
                c.shown.wrapping_add(1)
            }
        } else if c.shown == 0 {
            c.last
        } else {
            c.shown.wrapping_sub(1)
        },
        ..c
    }
}

/// The entry whose car is `car` (`cars` the entries' cars before the end mark), 0 when none
/// is.
pub fn find_car(cars: &[u32], car: u32) -> u32 {
    cars.iter().position(|&c| c == car).unwrap_or(0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons() {
        let mut s = Shown {
            car: 5,
            last_car: 5,
            picture: 0,
            page: 3,
            leave: true,
        };
        assert_eq!(
            press(Button::NextCar, &mut s),
            &[Call::Background, Call::Stream]
        );
        assert_eq!((s.car, s.leave), (0, false));
        s.leave = true;
        assert_eq!(
            press(Button::PreviousCar, &mut s),
            &[Call::Background, Call::Stream]
        );
        assert_eq!((s.car, s.leave), (5, false));
        press(Button::PreviousCar, &mut s);
        assert_eq!(s.car, 4);
        s.car = 0;
        press(Button::PreviousSlideshowCar, &mut s);
        assert_eq!(s.car, 5);
        press(Button::NextPage, &mut s);
        press(Button::PreviousPicture, &mut s);
        assert_eq!((s.page, s.picture), (0, 99));
        assert_eq!((find_car(&[4, 7, 9], 9), find_car(&[4], 9)), (2, 0));
    }

    #[test]
    fn specs() {
        assert_eq!(specs_list(2), Some((SpecsList::Events, true)));
        assert_eq!(specs_list(4), Some((SpecsList::Kind4, true)));
        assert_eq!(specs_list(5), Some((SpecsList::Kind5, false)));
        assert_eq!((specs_list(0), specs_list(6)), (None, None));
    }

    #[test]
    fn previous_page() {
        let mut s = Shown::default();
        assert_eq!(press(Button::PreviousPage, &mut s), &[Call::Background]);
        assert_eq!(s.page, 3);
        press(Button::PreviousPage, &mut s);
        assert_eq!(s.page, 2);
        s.page = u32::MAX;
        press(Button::PreviousPage, &mut s);
        assert_eq!(s.page, 3);
    }
}
