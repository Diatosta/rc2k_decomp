//! The car pages' pictures: the car whose pictures are shown, the page's background and the
//! slideshow's search for a picture that is there.

use crate::Invalid;

/// The car pages' background tables, one a page.
pub const PAGES: u32 = 4;

/// The cars with slideshow pictures: the first 0x18.
pub const SLIDESHOW_CARS: u32 = 0x18;

/// A car's pictures are numbered 0 .. 99.
pub const PICTURES: i32 = 100;

/// The car whose pictures stand for `car`: 0x21 counts as 0x17.
pub fn picture_car(car: u32) -> u32 {
    if car == 0x21 { 0x17 } else { car }
}

/// The background page. Refused: past the four.
pub fn background_page(page: u32) -> Result<u32, Invalid> {
    if page >= PAGES {
        return Err(Invalid("a page past the four"));
    }
    Ok(page)
}

/// The slideshow's car. Refused: one with no pictures.
pub fn slideshow_car(car: u32) -> Result<u32, Invalid> {
    if car >= SLIDESHOW_CARS {
        return Err(Invalid("a car with no slideshow"));
    }
    Ok(car)
}

/// The picture number after `number` the way looked (`back`: down, from 0 round to 99; else up,
/// from 99 round to 0; signed).
pub fn next_picture(number: u32, back: bool) -> u32 {
    let n = number as i32;
    let next = if back {
        let n = n.wrapping_sub(1);
        if n < 0 { PICTURES - 1 } else { n }
    } else {
        let n = n.wrapping_add(1);
        if n > PICTURES - 1 { 0 } else { n }
    };
    next as u32
}

/// A path's last two characters made "cx" (`.png` to `.pcx`). Refused: fewer than two.
pub fn pcx(path: &mut [u8]) -> Result<(), Invalid> {
    let n = path.len();
    if n < 2 {
        return Err(Invalid("a picture's path of fewer than two characters"));
    }
    path[n - 2..].copy_from_slice(b"cx");
    Ok(())
}

/// The slideshow's search from picture `start`: `tried(number)` (true: a picture taken); otherwise
/// the number steps (`moved` gets each next one, `EventState.shown.picture`) until it comes round
/// to `start`, when the last one tried is taken anyway. A start past 0 .. 99 never comes round
/// (kept).
pub fn search<E>(
    start: u32,
    back: bool,
    mut tried: impl FnMut(u32) -> Result<bool, E>,
    mut moved: impl FnMut(u32),
) -> Result<(), E> {
    let mut number = start;
    loop {
        if tried(number)? {
            return Ok(());
        }
        number = next_picture(number, back);
        moved(number);
        if number == start {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pictures() {
        assert_eq!(picture_car(0x21), 0x17);
        assert_eq!(picture_car(3), 3);
        assert!(background_page(4).is_err() && background_page(3).is_ok());
        assert!(slideshow_car(0x18).is_err() && slideshow_car(0x17).is_ok());
        assert_eq!(next_picture(0, true), 99);
        assert_eq!(next_picture(99, false), 0);
        assert_eq!(next_picture(5, false), 6);
        assert_eq!(next_picture(150, true), 149);
        let mut p = *b"slide.png";
        pcx(&mut p).unwrap();
        assert_eq!(&p, b"slide.pcx");
        assert!(pcx(&mut b"x".to_owned()).is_err());
    }

    #[test]
    fn searches() {
        let (mut tried, mut moved) = (Vec::new(), Vec::new());
        search::<()>(
            98,
            false,
            |n| {
                tried.push(n);
                Ok(n == 1)
            },
            |n| moved.push(n),
        )
        .unwrap();
        assert_eq!(tried, vec![98, 99, 0, 1]);
        assert_eq!(moved, vec![99, 0, 1]);
        let mut count = 0;
        search::<()>(
            7,
            true,
            |_| {
                count += 1;
                Ok(false)
            },
            |_| {},
        )
        .unwrap();
        assert_eq!(count, 100);
    }
}
