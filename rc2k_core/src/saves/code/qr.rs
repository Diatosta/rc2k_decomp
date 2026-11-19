//! The code as a QR code: the modules to draw, and a QR code read back out of a picture.
//!
//! The lowest error correction is used: the code is shown on a screen, scanned from a screen
//! or a camera frame, so there is no dirt or damage to correct and every module saved goes to
//! the text. The text is in the alphanumeric mode's letters, so a code of version 40 holds
//! 4296 of them.

use qrcode::{Color, EcLevel, QrCode};

/// How many letters the largest code holds in the alphanumeric mode at the lowest correction.
pub const MOST_LETTERS: usize = 4296;

/// A QR code's modules, row by row, true for dark; the quiet zone is not in them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Modules {
    pub side: usize,
    pub dark: Vec<bool>,
}

impl Modules {
    pub fn at(&self, x: usize, y: usize) -> bool {
        self.dark[y * self.side + x]
    }
}

/// The width of the quiet zone, in modules.
pub const QUIET: usize = 4;

/// The code for `text`, or `None` if it does not fit the largest.
pub fn encode(text: &str) -> Option<Modules> {
    let code = QrCode::with_error_correction_level(text.as_bytes(), EcLevel::L).ok()?;
    Some(Modules {
        side: code.width(),
        dark: code
            .to_colors()
            .into_iter()
            .map(|c| c == Color::Dark)
            .collect(),
    })
}

/// How many pixels a module takes so that the code with its quiet zone fits a square of `room`
/// pixels, whole pixels only (a module of a part of a pixel makes a code that does not scan).
/// `None` when even one pixel a module does not fit.
pub fn module_pixels(side: usize, room: usize) -> Option<usize> {
    let scale = room / (side + 2 * QUIET);
    (scale > 0).then_some(scale)
}

/// The code as an image of grey levels (0 dark, 255 light) with its quiet zone, a module
/// `scale` pixels square: (width, pixels).
pub fn render(modules: &Modules, scale: usize) -> (usize, Vec<u8>) {
    let width = (modules.side + 2 * QUIET) * scale;
    let mut pixels = vec![255u8; width * width];
    for y in 0..modules.side {
        for x in 0..modules.side {
            if modules.at(x, y) {
                for row in 0..scale {
                    let at = ((y + QUIET) * scale + row) * width + (x + QUIET) * scale;
                    pixels[at..at + scale].fill(0);
                }
            }
        }
    }
    (width, pixels)
}

/// The text of the first QR code in an image of grey levels (`width` x `height`), if one can
/// be read.
pub fn read(width: usize, height: usize, grey: &[u8]) -> Option<String> {
    if width == 0 || grey.len() < width * height {
        return None;
    }
    let mut image =
        rqrr::PreparedImage::prepare_from_greyscale(width, height, |x, y| grey[y * width + x]);
    image
        .detect_grids()
        .into_iter()
        .find_map(|grid| grid.decode().ok().map(|(_, text)| text))
}

/// How a camera frame had to be treated for its code to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seen {
    /// As the camera sent it.
    Plain,
    /// Mirrored left to right (a code shown in a mirror, or by a camera that mirrors).
    Mirrored,
    /// Light modules on a dark ground.
    Inverted,
}

/// The text of the first QR code in a camera frame of grey levels, read as it is, else mirrored
/// left to right, else with the light and dark swapped (the reader takes none of those on its
/// own); with how the frame had to be treated.
pub fn read_camera(width: usize, height: usize, grey: &[u8]) -> Option<(String, Seen)> {
    if width == 0 || grey.len() < width * height {
        return None;
    }
    if let Some(text) = read(width, height, grey) {
        return Some((text, Seen::Plain));
    }
    let mirrored: Vec<u8> = grey
        .chunks_exact(width)
        .flat_map(|row| row.iter().rev().copied())
        .collect();
    if let Some(text) = read(width, height, &mirrored) {
        return Some((text, Seen::Mirrored));
    }
    let inverted: Vec<u8> = grey.iter().map(|g| 255 - g).collect();
    read(width, height, &inverted).map(|text| (text, Seen::Inverted))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(n: usize) -> String {
        let alphabet = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";
        (0..n)
            .map(|i| char::from(alphabet[(i * i + 7 * i + i / 13) % 45]))
            .collect()
    }

    #[test]
    fn a_code_is_read_back_from_its_picture() {
        for n in [20, 300, 1500, 4000, MOST_LETTERS] {
            let text = sample(n);
            let modules = encode(&text).unwrap_or_else(|| panic!("{n} letters"));
            let scale = 3;
            let (width, pixels) = render(&modules, scale);
            assert_eq!(
                read(width, width, &pixels).as_deref(),
                Some(text.as_str()),
                "{n}"
            );
        }
    }

    #[test]
    fn too_long_a_text_has_no_code() {
        assert!(encode(&sample(MOST_LETTERS + 1)).is_none());
    }

    #[test]
    fn modules_take_whole_pixels() {
        assert_eq!(module_pixels(177, 1080), Some(5));
        assert_eq!(module_pixels(177, 300), Some(1));
        assert_eq!(module_pixels(177, 150), None);
    }

    /// How many camera frames (each a little different, as a hand-held camera's are) a code
    /// is looked for in.
    const FRAMES: u32 = 6;

    /// What reading each of a few parts' views gives, within `FRAMES` frames of each.
    fn parts_seen(view: super::super::qr_frames::View) -> Vec<Option<(String, Seen)>> {
        let texts =
            super::super::parts::split(&sample(3000), super::super::parts::Density::Small).unwrap();
        assert!(texts.len() >= 5);
        texts
            .iter()
            .step_by(2)
            .map(|t| {
                let modules = encode(t).unwrap();
                (0..FRAMES).find_map(|n| {
                    let view = super::super::qr_frames::View { frame: n, ..view };
                    let grey = super::super::qr_frames::frame(&modules, view);
                    let got = read_camera(view.width, view.height, &grey);
                    assert!(
                        got.as_ref().is_none_or(|(read, _)| read == t),
                        "a wrong text"
                    );
                    got
                })
            })
            .collect()
    }

    fn all_read(seen: &[Option<(String, Seen)>], how: Seen) -> bool {
        seen.iter()
            .all(|s| s.as_ref().is_some_and(|(_, h)| *h == how))
    }

    #[test]
    fn parts_are_read_from_a_720p_camera_picture() {
        use super::super::qr_frames::View;
        let mut view = View::plain(1280, 720, 55);
        view.degrees = 4.0;
        view.blur = 1;
        view.noise = 12;
        assert!(all_read(&parts_seen(view), Seen::Plain));
    }

    #[test]
    fn parts_are_read_from_a_1080p_camera_picture_even_from_afar() {
        use super::super::qr_frames::View;
        let mut view = View::plain(1920, 1080, 40);
        view.degrees = -6.0;
        view.blur = 1;
        view.noise = 12;
        assert!(all_read(&parts_seen(view), Seen::Plain));
    }

    #[test]
    fn parts_are_read_from_a_vga_picture_when_close() {
        use super::super::qr_frames::View;
        let mut view = View::plain(640, 480, 85);
        view.blur = 1;
        view.noise = 8;
        assert!(all_read(&parts_seen(view), Seen::Plain));
    }

    /// What reading each of a few dense parts' views gives, within `FRAMES` frames of each.
    fn dense_parts_seen(view: super::super::qr_frames::View) -> Vec<Option<(String, Seen)>> {
        use super::super::parts::{Density, split};
        let texts = split(&sample(7000), Density::Dense).unwrap();
        assert!(texts.len() >= 3);
        texts
            .iter()
            .map(|t| {
                let modules = encode(t).unwrap();
                (0..FRAMES).find_map(|n| {
                    let view = super::super::qr_frames::View { frame: n, ..view };
                    let grey = super::super::qr_frames::frame(&modules, view);
                    let got = read_camera(view.width, view.height, &grey);
                    assert!(
                        got.as_ref().is_none_or(|(read, _)| read == t),
                        "a wrong text"
                    );
                    got
                })
            })
            .collect()
    }

    #[test]
    fn dense_parts_are_read_from_a_1080p_camera_picture() {
        use super::super::qr_frames::View;
        let mut view = View::plain(1920, 1080, 60);
        view.degrees = 3.0;
        view.blur = 1;
        view.noise = 8;
        assert!(dense_parts_seen(view).iter().all(Option::is_some));
    }

    #[test]
    fn dense_parts_are_read_from_a_4k_camera_picture_from_afar() {
        use super::super::qr_frames::View;
        let mut view = View::plain(3840, 2160, 35);
        view.degrees = -4.0;
        view.blur = 1;
        view.noise = 8;
        assert!(dense_parts_seen(view).iter().all(Option::is_some));
    }

    #[test]
    fn a_mirrored_picture_is_read() {
        use super::super::qr_frames::View;
        let mut view = View::plain(1280, 720, 60);
        view.degrees = 3.0;
        view.blur = 1;
        view.mirrored = true;
        // The reader takes some mirrored codes as they are; the rest need the mirrored try.
        let seen = parts_seen(view);
        assert!(seen.iter().all(Option::is_some));
        assert!(
            seen.iter()
                .any(|s| s.as_ref().is_some_and(|(_, h)| *h == Seen::Mirrored))
        );
    }

    #[test]
    fn an_inverted_picture_is_read_inverted() {
        use super::super::qr_frames::View;
        let mut view = View::plain(1280, 720, 60);
        view.noise = 10;
        view.inverted = true;
        assert!(all_read(&parts_seen(view), Seen::Inverted));
    }

    #[test]
    fn a_picture_turned_upside_down_is_read() {
        use super::super::qr_frames::View;
        let mut view = View::plain(1280, 720, 60);
        view.degrees = 180.0;
        view.blur = 1;
        assert!(all_read(&parts_seen(view), Seen::Plain));
    }

    #[test]
    fn a_far_away_code_in_a_vga_picture_is_not_read() {
        use super::super::qr_frames::View;
        let mut view = View::plain(640, 480, 35);
        view.blur = 1;
        view.noise = 12;
        assert!(parts_seen(view).iter().all(Option::is_none));
    }

    /// Writes a camera picture of a part of a made-up code as a binary PGM, for trying the
    /// scan page with a stand-in camera: `RC2K_SAMPLE_PGM` names the file.
    #[test]
    #[ignore = "writes a file for a manual check"]
    fn write_sample_picture() {
        use super::super::qr_frames::{View, frame};
        let Some(path) = std::env::var_os("RC2K_SAMPLE_PGM") else {
            return;
        };
        let texts = super::super::parts::split(
            &format!("RC2K:{}", sample(3000)),
            super::super::parts::Density::Small,
        )
        .unwrap();
        let modules = encode(&texts[0]).unwrap();
        let mut view = View::plain(1280, 720, 60);
        view.degrees = 3.0;
        view.blur = 1;
        view.noise = 8;
        let mut bytes = b"P5 1280 720 255\n".to_vec();
        bytes.extend(frame(&modules, view));
        std::fs::write(path, bytes).unwrap();
    }
}
