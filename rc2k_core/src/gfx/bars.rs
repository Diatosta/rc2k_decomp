//! Bars filled to a percentage (converted 2026-10-21 from `gfx::blit_bar` 0x46B574 and
//! `gfx::blit_bar_marker` 0x46B5FC), as the six bar globals describe them
//! (`dword_755224` ..).

pub use crate::screens::screen::{BAR_HAS_NO_LENGTH, Bar};

/// What a bar is drawn with.
pub trait BarCanvas {
    type Error;
    /// A sprite of the sprite table.
    fn sprite(&mut self, glyph: u32, x: i32, y: i32, palette: u32) -> Result<(), Self::Error>;
    /// The clip rectangle's right edge, and it set.
    fn clip_right(&mut self) -> i32;
    fn set_clip_right(&mut self, x: i32);
    /// `seconds` (0x7559AC): the fill's end.
    fn keep_fill_end(&mut self, x: i32);
    /// The bar's x (`dword_755224`) set: the marker's place.
    fn set_bar_x(&mut self, x: i32);
    /// The game's fatal error 0x80701010.
    fn no_length(&mut self) -> Self::Error;
}

/// Pixels from x to where `percent` of the width ends (an unsigned division by 100); a
/// width x percent of 0 has none.
pub fn length(bar: &Bar) -> Option<i32> {
    let product = bar.width.wrapping_mul(bar.percent);
    (product != 0).then_some((product as u32 / 100) as i32)
}

/// The bar (nothing at 0 percent): the fill sprite from x, cut at the fill's end by the
/// clip rectangle (put back afterwards), then the end sprite there; both with the end's x
/// as their palette.
pub fn bar<C: BarCanvas>(c: &mut C, bar: &Bar) -> Result<(), C::Error> {
    if bar.percent == 0 {
        return Ok(());
    }
    let Some(length) = length(bar) else {
        return Err(c.no_length());
    };
    let end = bar.x.wrapping_add(length);
    let saved = c.clip_right();
    c.set_clip_right(end);
    c.keep_fill_end(end);
    let fill = c.sprite(bar.fill_glyph, bar.x, bar.y, end as u32);
    c.set_clip_right(saved);
    fill?;
    c.sprite(bar.end_glyph, end, bar.y, end as u32)
}

/// The marker (the fill sprite) at where the percentage ends, the bar's x moved there and
/// the x as its palette; at 0 percent or less at the bar's x with `palette` (the caller's
/// ebx).
pub fn marker<C: BarCanvas>(c: &mut C, bar: &Bar, palette: u32) -> Result<(), C::Error> {
    if bar.percent <= 0 {
        return c.sprite(bar.fill_glyph, bar.x, bar.y, palette);
    }
    let Some(length) = length(bar) else {
        return Err(c.no_length());
    };
    let x = bar.x.wrapping_add(length);
    c.set_bar_x(x);
    c.sprite(bar.fill_glyph, x, bar.y, x as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Record(Vec<String>, i32);

    impl BarCanvas for Record {
        type Error = &'static str;
        fn sprite(&mut self, g: u32, x: i32, y: i32, p: u32) -> Result<(), &'static str> {
            self.0.push(format!("{g} {x} {y} {p} clip {}", self.1));
            Ok(())
        }
        fn clip_right(&mut self) -> i32 {
            self.1
        }
        fn set_clip_right(&mut self, x: i32) {
            self.1 = x;
        }
        fn keep_fill_end(&mut self, x: i32) {
            self.0.push(format!("end {x}"));
        }
        fn set_bar_x(&mut self, x: i32) {
            self.0.push(format!("x {x}"));
        }
        fn no_length(&mut self) -> &'static str {
            "no length"
        }
    }

    #[test]
    fn bars() {
        let b = Bar {
            x: 10,
            y: 20,
            width: 200,
            fill_glyph: 1,
            end_glyph: 2,
            percent: 50,
        };
        let mut c = Record(Vec::new(), 799);
        bar(&mut c, &b).unwrap();
        assert_eq!(
            c.0,
            ["end 110", "1 10 20 110 clip 110", "2 110 20 110 clip 799"]
        );
        let mut c = Record(Vec::new(), 799);
        marker(&mut c, &b, 9).unwrap();
        assert_eq!(c.0, ["x 110", "1 110 20 110 clip 799"]);
        let mut c = Record::default();
        marker(&mut c, &Bar { percent: 0, ..b }, 9).unwrap();
        assert_eq!(c.0, ["1 10 20 9 clip 0"]);
        assert_eq!(bar(&mut c, &Bar { width: 0, ..b }), Err("no length"));
    }
}
