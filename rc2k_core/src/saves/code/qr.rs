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
}
