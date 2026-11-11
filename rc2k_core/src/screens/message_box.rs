//! The "not in this version" box (from `screens::draw_message_box`
//! 0x415EA6): string 0x1FE in a box in the middle of the surface, 0x32 wider and 0x19
//! higher than the string measures, the string drawn centred in another font.

/// The string shown.
pub const STRING: u32 = 0x1FE;
/// The colours the box is filled and framed with (through the fonts' palettes).
pub const FILL: u32 = 1;
pub const FRAME: u32 = 0xA;

/// The box and where the string goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MessageBox {
    /// x, y, width, height (`dword_52983C` ..).
    pub rect: [u32; 4],
    pub text_x: i32,
    pub text_y: i32,
}

/// The box for a string that measures `width` and ends at `y`, on a surface `screen_w` x
/// `screen_h`, the drawing font `font_height` high (all halved arithmetically).
pub fn layout(width: u32, y: i32, screen_w: u32, screen_h: u32, font_height: u32) -> MessageBox {
    let w = width.wrapping_add(0x32);
    let h = (y as u32).wrapping_add(0x19);
    MessageBox {
        rect: [
            (screen_w.wrapping_sub(w) as i32 >> 1) as u32,
            (screen_h.wrapping_sub(h) as i32 >> 1) as u32,
            w,
            h,
        ],
        text_x: (screen_w as i32) >> 1,
        text_y: (screen_h.wrapping_sub(font_height) as i32 >> 1) - 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centred() {
        let b = layout(200, 20, 640, 480, 16);
        assert_eq!(b.rect, [195, 217, 250, 45]);
        assert_eq!((b.text_x, b.text_y), (320, 231));
        let b = layout(700, 20, 640, 480, 16);
        assert_eq!(b.rect[0], (-55i32) as u32);
    }
}
