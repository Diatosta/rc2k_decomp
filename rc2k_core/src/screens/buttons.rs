//! The menu's buttons (converted 2026-10-20 from `screens::draw_button` 0x4688A1,
//! `draw_button_highlighted` 0x468BBE and `draw_button_alt` 0x468EDD): a left end, a run of
//! 32-pixel middle pieces and a right end. The pieces are placed here; drawing them is the
//! caller's (`blit(sprite, x, y, palette)`, the palette always the button's x).

/// Where x is: the button's middle, its left edge, or where its right end piece goes.
pub const ALIGN_CENTRE: u32 = 1;
pub const ALIGN_LEFT: u32 = 2;
pub const ALIGN_RIGHT: u32 = 3;

/// How one of the three functions goes wrong (the original's slips, kept).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kind {
    /// `draw_button`: right-aligned, from the second whole middle piece on it goes on in the
    /// centred way's loop (rightwards, a second right end); with a width a multiple of 32
    /// it draws a second, centred button after the left end.
    pub slips: bool,
    /// `draw_button_alt`: left-aligned, the whole middle pieces take their y, and the last
    /// shorter one its x, from `draw_button`'s work (`Borrowed`).
    pub borrows: bool,
}

/// `draw_button`'s work y and x, which `Kind::borrows` reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Borrowed {
    pub y: i32,
    pub x: i32,
}

/// Why a button was not finished.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error<E> {
    /// A width of 0 (after the first piece): the original's fatal error 0x80701010.
    NoLength,
    /// An alignment not 1 ..= 3: its fatal error 0x8070202n.
    BadAlign,
    /// The drawing's own.
    Draw(E),
}

struct Pen<'a, E> {
    x: i32,
    y: i32,
    palette: u32,
    blit: &'a mut dyn FnMut(u32, i32, i32, u32) -> Result<(), E>,
}

impl<E> Pen<'_, E> {
    fn put(&mut self, sprite: u32) -> Result<(), Error<E>> {
        self.put_at(sprite, self.x, self.y)
    }

    fn put_at(&mut self, sprite: u32, x: i32, y: i32) -> Result<(), Error<E>> {
        (self.blit)(sprite, x, y, self.palette).map_err(Error::Draw)
    }

    fn step(&mut self, by: i32) {
        self.x = self.x.wrapping_add(by);
    }

    fn rightwards(
        &mut self,
        wholes: u32,
        rest: u32,
        middle: u32,
        right: u32,
    ) -> Result<(), Error<E>> {
        for _ in 0..wholes {
            self.put(middle)?;
            self.step(32);
        }
        if rest != 0 {
            self.step(rest as i32 - 32);
            self.put(middle)?;
            self.step(32);
        }
        self.put(right)
    }

    fn left_to_right(
        &mut self,
        width: u32,
        [left, right, middle]: [u32; 3],
    ) -> Result<(), Error<E>> {
        self.put(left)?;
        self.step(0x22);
        if width == 0 {
            return Err(Error::NoLength);
        }
        self.rightwards(width / 32, width % 32, middle, right)
    }

    fn left_to_right_borrowing(
        &mut self,
        width: u32,
        [left, right, middle]: [u32; 3],
        borrowed: Borrowed,
    ) -> Result<(), Error<E>> {
        self.put(left)?;
        self.step(0x22);
        if width == 0 {
            return Err(Error::NoLength);
        }
        for _ in 0..width / 32 {
            self.put_at(middle, self.x, borrowed.y)?;
            self.step(32);
        }
        let rest = width % 32;
        if rest != 0 {
            self.step(rest as i32 - 32);
            self.put_at(middle, borrowed.x, self.y)?;
            self.step(32);
        }
        self.put(right)
    }

    fn button(
        &mut self,
        kind: Kind,
        width: u32,
        align: u32,
        sprites: [u32; 3],
        borrowed: Borrowed,
    ) -> Result<(), Error<E>> {
        let [left, right, middle] = sprites;
        let to_centre = ((width >> 1).wrapping_add(0x22) as i32).wrapping_neg();
        match align {
            ALIGN_LEFT if kind.borrows => self.left_to_right_borrowing(width, sprites, borrowed),
            ALIGN_LEFT => self.left_to_right(width, sprites),
            ALIGN_CENTRE => {
                self.step(to_centre);
                self.left_to_right(width, sprites)
            }
            ALIGN_RIGHT => {
                self.put(right)?;
                self.step(-32);
                if width == 0 {
                    return Err(Error::NoLength);
                }
                let (wholes, rest) = (width / 32, width % 32);
                if wholes != 0 {
                    self.put(middle)?;
                    self.step(-32);
                    if kind.slips {
                        if wholes > 1 {
                            return self.rightwards(wholes - 1, rest, middle, right);
                        }
                    } else {
                        for _ in 1..wholes {
                            self.put(middle)?;
                            self.step(-32);
                        }
                    }
                }
                if rest != 0 {
                    self.step(32 - rest as i32);
                    self.put(middle)?;
                    self.step(-0x22);
                    return self.put(left);
                }
                self.step(-2);
                self.put(left)?;
                if kind.slips {
                    self.step(to_centre);
                    return self.left_to_right(width, sprites);
                }
                Ok(())
            }
            _ => Err(Error::BadAlign),
        }
    }
}

/// A button `width` wide (its middle pieces) at x, y by `align`, of the sprites (left end,
/// right end, middle). Returns where the pen ended (the original's work x) and how
/// drawing went; pieces before an error are drawn.
#[allow(clippy::too_many_arguments)]
pub fn draw<E>(
    kind: Kind,
    width: u32,
    x: i32,
    y: i32,
    align: u32,
    sprites: [u32; 3],
    borrowed: Borrowed,
    blit: &mut dyn FnMut(u32, i32, i32, u32) -> Result<(), E>,
) -> (i32, Result<(), Error<E>>) {
    let mut pen = Pen {
        x,
        y,
        palette: x as u32,
        blit,
    };
    let drawn = pen.button(kind, width, align, sprites, borrowed);
    (pen.x, drawn)
}

/// A text button's width: its text's, measured, and 12.
pub fn text_width(measured: u32) -> u32 {
    measured.wrapping_add(0xC)
}

#[cfg(test)]
mod tests {
    use super::*;

    type Pieces = (i32, Vec<(u32, i32)>, Result<(), Error<()>>);

    fn pieces(kind: Kind, width: u32, align: u32) -> Pieces {
        let mut drawn = Vec::new();
        let (x, r) = draw(
            kind,
            width,
            100,
            50,
            align,
            [1, 2, 3],
            Borrowed::default(),
            &mut |s, x, _, _| {
                drawn.push((s, x));
                Ok(())
            },
        );
        (x, drawn, r)
    }

    #[test]
    fn buttons() {
        let plain = Kind {
            slips: false,
            borrows: false,
        };
        // Left: end at 100, middles at 0x86 and 0xA6 (40 = 32 + 8: the last moved back 24), end.
        let (_, d, r) = pieces(plain, 40, ALIGN_LEFT);
        assert!(r.is_ok());
        assert_eq!(d, [(1, 100), (3, 0x86), (3, 0x8E), (2, 0xAE)]);
        assert_eq!(pieces(plain, 0, ALIGN_LEFT).2, Err(Error::NoLength));
        assert_eq!(pieces(plain, 32, 7).2, Err(Error::BadAlign));
        // `draw_button`'s slip: right-aligned with a multiple of 32, a second button.
        let slipping = Kind {
            slips: true,
            borrows: false,
        };
        let (_, d, _) = pieces(slipping, 32, ALIGN_RIGHT);
        assert_eq!(d.len(), 3 + 3);
        assert_eq!(text_width(20), 32);
    }
}
