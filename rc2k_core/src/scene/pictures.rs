//! The system textures' pictures (from `scene::load_texture_picture`
//! 0x42578E, `scene::load_changed_texture_picture` 0x42574B and
//! `scene::reload_system_textures` 0x4254B4): which picture a texture wants and whether it
//! is to be loaded, the picture's file names (`tx<number>.pcx`, four decimal digits, and the
//! one numbered one higher) and folder, and the size a picture is read at.

use crate::scene::textures::{PictureFolder, picture_folder};

/// The four decimal digits of a picture's number as they go into its name (each digit
/// plus '0', the thousands first: a number past 9999 gives a byte past '9').
pub fn digits(number: u32) -> [u8; 4] {
    [
        (number / 1000) as u8,
        (number % 1000 / 100) as u8,
        (number % 100 / 10) as u8,
        (number % 10) as u8,
    ]
    .map(|digit| digit.wrapping_add(b'0'))
}

/// The folders' names in the work block, in order: `files\a`, `files\m`, `files\b`,
/// `files\b\gravel`, `files\g\frnt`, `files\s\extr`.
pub const FOLDERS: usize = 6;
/// The gravel folder's place among them.
pub const GRAVEL: usize = 3;

/// The digits written into each folder's name and second name (`None`: left as it is).
/// The `files\m` name gets its third digit twice and the `files\s\extr` second name keeps
/// its `xxxx` (the original's slips, kept).
pub fn names(number: u32) -> [([u8; 4], Option<[u8; 4]>); FOLDERS] {
    let (first, second) = (digits(number), digits(number.wrapping_add(1)));
    std::array::from_fn(|n| {
        let mut name = first;
        if n == 1 {
            name[3] = name[2];
        }
        (name, (n != 5).then_some(second))
    })
}

/// A folder's place among the names.
pub fn folder_place(folder: PictureFolder) -> usize {
    match folder {
        PictureFolder::A => 0,
        PictureFolder::M => 1,
        PictureFolder::B => 2,
        PictureFolder::Front => 4,
        PictureFolder::Extra => 5,
    }
}

/// The car a picture 1000 .. 1999 is for: the texture less the cars' first texture (kind
/// 0x100's), over 8, when that is under the cars' count (unsigned). Both are asked for
/// only for such a picture.
pub fn picture_car(
    number: u32,
    texture: u32,
    first_car_texture: impl FnOnce() -> u32,
    cars: impl FnOnce() -> u32,
) -> Option<u32> {
    if !(1000..2000).contains(&number) {
        return None;
    }
    let car = texture.wrapping_sub(first_car_texture()) >> 3;
    (car < cars()).then_some(car)
}

/// Whether a car's gravel value takes its picture from `files\b\gravel`
/// when the file is there: 3 or more, by the difference's sign as the original has it.
pub fn takes_gravel(gravel: u32) -> bool {
    gravel.wrapping_sub(3) as i32 >= 0
}

/// `load_texture_picture`'s folder: by the number (`texture_picture_folder`), or the
/// gravel one.
pub fn folder(number: u32, gravel: bool) -> usize {
    if gravel {
        GRAVEL
    } else {
        folder_place(picture_folder(number))
    }
}

/// `load_changed_texture_picture`: the picture to load into a texture, unless it wants
/// none (-1 or 0) or holds it already (`loaded` asked only then).
pub fn changed_picture(wanted: u32, loaded: impl FnOnce() -> u32) -> Option<u32> {
    (wanted != u32::MAX && wanted != 0 && wanted != loaded()).then_some(wanted)
}

/// `reload_system_textures`: the picture a kind's first texture wants (its kind's picture,
/// 0 made -1).
pub fn kind_picture(picture: u32) -> u32 {
    if picture == 0 { u32::MAX } else { picture }
}

/// The size (height, width) a picture is read at: 0x100 square for picture 0x3DE and
/// 0x706 .. 0x7CF, else the scene's.
pub fn read_size(picture: u32, scene: [u32; 2]) -> [u32; 2] {
    if picture == 0x3DE || (0x706..=0x7CF).contains(&picture) {
        [0x100, 0x100]
    } else {
        [scene[1], scene[0]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pictures() {
        assert_eq!(&digits(1234), b"1234");
        assert_eq!(digits(10_000), [b':', b'0', b'0', b'0']);
        let n = names(900);
        assert_eq!((&n[0].0, n[0].1), (b"0900", Some(*b"0901")));
        assert_eq!(&n[1].0, b"0900");
        assert_eq!(&names(1234)[1].0, b"1233");
        assert_eq!(n[5].1, None);
        assert_eq!(
            (folder(900, false), folder(901, false), folder(902, false)),
            (1, 1, 0)
        );
        assert_eq!(
            (
                folder(1500, false),
                folder(2999, false),
                folder(3000, false)
            ),
            (2, 4, 5)
        );
        assert_eq!(folder(1500, true), GRAVEL);
        let car = |number, texture| picture_car(number, texture, || 0x100, || 4);
        assert_eq!(car(1500, 0x118), Some(3));
        assert_eq!(car(1500, 0x120), None);
        assert_eq!(car(1500, 0xF8), None);
        assert_eq!(
            picture_car(999, 0x100, || unreachable!(), || unreachable!()),
            None
        );
        assert!(takes_gravel(3) && !takes_gravel(2) && !takes_gravel(0x8000_0003));
        // by the sign of the difference: 0x80000002 less 3 is positive
        assert!(takes_gravel(0x8000_0002));
        assert_eq!(changed_picture(5, || 6), Some(5));
        assert_eq!(changed_picture(5, || 5), None);
        assert_eq!(changed_picture(0, || unreachable!()), None);
        assert_eq!(changed_picture(u32::MAX, || unreachable!()), None);
        assert_eq!((kind_picture(0), kind_picture(7)), (u32::MAX, 7));
        assert_eq!(read_size(0x3DE, [640, 480]), [0x100, 0x100]);
        assert_eq!(read_size(0x7D0, [640, 480]), [480, 640]);
    }
}
