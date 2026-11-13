//! Bars filled to a percentage, as the six bar globals describe them (`BarVars.x` ..).

pub use crate::screens::screen::{BAR_HAS_NO_LENGTH, Bar};

/// What a bar is drawn with.
pub trait BarCanvas {
    type Error;
    /// A sprite of the sprite table.
    fn sprite(&mut self, glyph: u32, x: i32, y: i32) -> Result<(), Self::Error>;
    /// The clip rectangle's right edge (read, and set by `set_clip_right`).
    fn clip_right(&mut self) -> i32;
    fn set_clip_right(&mut self, x: i32);
    /// The fill's end kept (in `seconds`).
    fn keep_fill_end(&mut self, x: i32);
    /// The bar's x set: the marker's place.
    fn set_bar_x(&mut self, x: i32);
    /// The fatal error for a bar with no length (`BAR_HAS_NO_LENGTH`).
    fn no_length(&mut self) -> Self::Error;
}

/// Pixels from x to where `percent` of the width ends (an unsigned division by 100); a
/// width x percent of 0 has none.
pub fn length(bar: &Bar) -> Option<i32> {
    let product = bar.width.wrapping_mul(bar.percent);
    (product != 0).then_some((product as u32 / 100) as i32)
}

/// The bar (nothing at 0 percent): the fill sprite from x, cut at the fill's end by the
/// clip rectangle (put back afterwards), then the end sprite there.
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
    let fill = c.sprite(bar.fill_glyph, bar.x, bar.y);
    c.set_clip_right(saved);
    fill?;
    c.sprite(bar.end_glyph, end, bar.y)
}

/// The marker (the fill sprite) where the percentage ends, the bar's x moved there; at 0 percent
/// or less, at the bar's x.
pub fn marker<C: BarCanvas>(c: &mut C, bar: &Bar) -> Result<(), C::Error> {
    if bar.percent <= 0 {
        return c.sprite(bar.fill_glyph, bar.x, bar.y);
    }
    let Some(length) = length(bar) else {
        return Err(c.no_length());
    };
    let x = bar.x.wrapping_add(length);
    c.set_bar_x(x);
    c.sprite(bar.fill_glyph, x, bar.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Record(Vec<String>, i32);

    impl BarCanvas for Record {
        type Error = &'static str;
        fn sprite(&mut self, g: u32, x: i32, y: i32) -> Result<(), &'static str> {
            self.0.push(format!("{g} {x} {y} clip {}", self.1));
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
        assert_eq!(c.0, ["end 110", "1 10 20 clip 110", "2 110 20 clip 799"]);
        let mut c = Record(Vec::new(), 799);
        marker(&mut c, &b).unwrap();
        assert_eq!(c.0, ["x 110", "1 110 20 clip 799"]);
        let mut c = Record::default();
        marker(&mut c, &Bar { percent: 0, ..b }).unwrap();
        assert_eq!(c.0, ["1 10 20 clip 0"]);
        assert_eq!(bar(&mut c, &Bar { width: 0, ..b }), Err("no length"));
    }
}
