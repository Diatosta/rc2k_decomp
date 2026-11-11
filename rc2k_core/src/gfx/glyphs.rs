//! The sprite and font builder's work on one sheet: `gfx::build_glyphs` calls it; what
//! the original's functions under `gfx::build_glyphs` do with a loaded sheet is here, in their
//! order. A sheet is a picture with glyphs drawn on a backdrop. The builder finds each
//! glyph's rectangle, codes it, fills the rectangle with the
//! backdrop and goes on looking; the coded glyphs become a table of offsets followed by
//! the glyph records, which is appended to the table built so far.
//!
//! The sheet is read through `SheetPixels` (in the game, the row table `g_rowTable`), as
//! bytes that are palette numbers or as dwords of four components, by bit 1 of the
//! builder's flags. From `gfx/glyph_builder.rs`.
//! A glyph is coded as the blitter reads it (CLAUDE.md, section 11): a head with the offsets
//! and the size left after blank edges are cut off, then for every row runs of count, gap
//! and pixels, a count of 0 ending the row. Blank is the backdrop or the transparent
//! colour. Without bit 0 of the flags a glyph is its pixels and nothing else.

use crate::Invalid;
use crate::gfx::pixels::PixelFormat;

/// A sheet's pixels by place: bytes (palette numbers) or dwords (components). Places
/// outside the sheet may be asked for (the original reads wherever its row table leads).
pub trait SheetPixels {
    fn byte(&self, x: u32, y: u32) -> u8;
    fn set_byte(&mut self, x: u32, y: u32, value: u8);
    fn dword(&self, x: u32, y: u32) -> u32;
    fn set_dword(&mut self, x: u32, y: u32, value: u32);
}

/// The builder's flags (a game-wide setting).
pub const RUN_LENGTHS: u32 = 1;
pub const DIRECT_COLOUR: u32 = 2;
pub const WIDE_HEAD: u32 = 4;
pub const ONE_FILE: u32 = 0x10;
pub const WITH_ALPHA: u32 = 0x20;

/// One glyph found on a sheet: its rectangle there, and its record's bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Glyph {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    /// Bits for what did not fit a byte (the original only logs them).
    pub warnings: u32,
    pub data: Vec<u8>,
}

/// A sheet as loaded, and how its glyphs are to be coded.
pub struct Sheet<'a> {
    pub flags: u32,
    /// Bytes of a coded pixel: 1, 2 or 4.
    pub pixel_size: u32,
    pub width: u32,
    pub height: u32,
    pub backdrop: u32,
    pub transparent: u32,
    /// Makes a direct-colour pixel from its components; not used for palette numbers.
    pub convert: &'a mut dyn FnMut(u32) -> u32,
    /// The sheet's pixels are to get an alpha made from their neighbours: the alpha flag
    /// is set and no alpha sheet was loaded. Each glyph's rectangle is then one pixel
    /// larger on every side.
    pub soften: bool,
    pub pixels: &'a mut dyn SheetPixels,
}

impl Sheet<'_> {
    /// The pixel at a place, wherever the row table and the place lead.
    fn get(&self, x: u32, y: u32) -> u32 {
        if self.flags & DIRECT_COLOUR != 0 {
            self.pixels.dword(x, y)
        } else {
            self.pixels.byte(x, y) as u32
        }
    }

    /// The backdrop put at a place.
    fn erase(&mut self, x: u32, y: u32) {
        if self.flags & DIRECT_COLOUR != 0 {
            self.pixels.set_dword(x, y, self.backdrop);
        } else {
            self.pixels.set_byte(x, y, self.backdrop as u8);
        }
    }

    /// Whether a place is on the sheet and not backdrop.
    fn is_drawn(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height && self.get(x, y) != self.backdrop
    }

    /// In a rectangle of a palette sheet, blank pixels become 0, and then
    /// every pixel's top four bits say how much of it and its eight neighbours inside
    /// the rectangle is drawn (a low nibble that is not 0): 3 for itself, 2 for each
    /// beside, above and below, 1 for each diagonal one.
    fn soften(&mut self, x: u32, y: u32, width: u32, height: u32) {
        let (last_column, last_row) = (x + width - 1, y + height - 1);
        for row in y..=last_row {
            for column in x..=last_column {
                if self.is_blank(self.pixels.byte(column, row) as u32) {
                    self.pixels.set_byte(column, row, 0);
                }
            }
        }
        for row in y..=last_row {
            for column in x..=last_column {
                let pixels = &*self.pixels;
                let drawn = |column: u32, row: u32| pixels.byte(column, row) & 0x0F != 0;
                let mut weight = 0u8;
                let other_row = |row: u32, weight: &mut u8| {
                    if column != x && drawn(column - 1, row) {
                        *weight += 1;
                    }
                    if column != last_column && drawn(column + 1, row) {
                        *weight += 1;
                    }
                    if drawn(column, row) {
                        *weight += 2;
                    }
                };
                if row != y {
                    other_row(row - 1, &mut weight);
                }
                if row != last_row {
                    other_row(row + 1, &mut weight);
                }
                if column != x && drawn(column - 1, row) {
                    weight += 2;
                }
                if column != last_column && drawn(column + 1, row) {
                    weight += 2;
                }
                if drawn(column, row) {
                    weight += 3;
                }
                let pixel = self.pixels.byte(column, row);
                self.pixels.set_byte(column, row, pixel | weight << 4);
            }
        }
    }

    fn is_blank(&self, pixel: u32) -> bool {
        pixel == self.transparent || pixel == self.backdrop
    }

    /// From a place in a row with `left` + 1 pixels to go, the blank pixels
    /// and then the drawn ones; the place and the count move past both.
    fn scan(&self, x: &mut u32, y: u32, left: &mut i32) -> (u32, u32) {
        let (mut blank, mut drawn) = (0u32, 0u32);
        if *left < 0 {
            return (0, 0);
        }
        while self.is_blank(self.get(*x, y)) {
            *x = x.wrapping_add(1);
            blank += 1;
            *left -= 1;
            if *left < 0 {
                return (blank, 0);
            }
        }
        while !self.is_blank(self.get(*x, y)) {
            *x = x.wrapping_add(1);
            drawn += 1;
            *left -= 1;
            if *left < 0 {
                break;
            }
        }
        (blank, drawn)
    }

    /// A count or a gap, as wide as a pixel.
    fn put_unit(&self, data: &mut Vec<u8>, value: u32) {
        data.extend(&value.to_le_bytes()[..self.pixel_size as usize]);
    }

    /// A pixel, converted unless it is a palette number.
    fn put_pixel(&mut self, data: &mut Vec<u8>, pixel: u32) {
        if self.pixel_size == 1 {
            data.push(pixel as u8);
        } else {
            let converted = (self.convert)(pixel);
            data.extend(&converted.to_le_bytes()[..self.pixel_size as usize]);
        }
    }

    /// The record of the glyph in a rectangle.
    fn coded(&mut self, x: u32, y: u32, width: u32, height: u32) -> Result<Glyph, Invalid> {
        let mut glyph = Glyph {
            x,
            y,
            width,
            height,
            warnings: 0,
            data: Vec::new(),
        };
        if self.flags & RUN_LENGTHS == 0 {
            for row in 0..height.max(1) {
                for column in 0..width.max(1) {
                    let pixel = self.get(x.wrapping_add(column), y.wrapping_add(row));
                    self.put_pixel(&mut glyph.data, pixel);
                }
            }
            return Ok(glyph);
        }

        // The registers as the original has them: the place, and counts that are one
        // less than what is left.
        let (mut left_edge, mut top) = (x, y);
        let mut columns = width.wrapping_sub(1) as i32;
        let mut rows = height.wrapping_sub(1);
        // Blank rows off the top, then off the bottom; the last row left is not looked at.
        let mut top_cut = 0u32;
        while rows != 0 {
            let (mut at, mut left) = (left_edge, columns);
            if self.scan(&mut at, top, &mut left).1 != 0 {
                let mut bottom = top.wrapping_add(rows);
                while rows != 0 {
                    let (mut at, mut left) = (left_edge, columns);
                    if self.scan(&mut at, bottom, &mut left).1 != 0 {
                        break;
                    }
                    bottom = bottom.wrapping_sub(1);
                    rows -= 1;
                }
                break;
            }
            top = top.wrapping_add(1);
            top_cut += 1;
            rows -= 1;
        }
        // The fewest blank pixels at the start of a row, and at its end.
        let (mut left_cut, mut right_cut) = (u32::MAX, u32::MAX);
        for row in 0..=rows {
            let (mut at, mut left) = (left_edge, columns);
            let (mut blank, mut drawn) = self.scan(&mut at, top.wrapping_add(row), &mut left);
            left_cut = left_cut.min(blank);
            while drawn != 0 {
                (blank, drawn) = self.scan(&mut at, top.wrapping_add(row), &mut left);
            }
            right_cut = right_cut.min(blank);
        }
        left_edge = left_edge.wrapping_add(left_cut);
        let (cut, borrowed) = (columns as u32).overflowing_sub(left_cut);
        let cut = if borrowed {
            cut
        } else {
            cut.wrapping_sub(right_cut)
        };
        let (cut_width, cut_height) = (cut.wrapping_add(1), rows.wrapping_add(1));
        columns = cut_width.wrapping_sub(1) as i32;

        // The size of the coded rows; and what the last scan found, which the original
        // looks at afterwards.
        let wide = self.flags & WIDE_HEAD != 0;
        let mut units = 0u32;
        let mut last;
        let mut row = 0u32;
        let mut rows_left = cut_height.wrapping_sub(1) as i32;
        'rows: loop {
            let (mut at, mut left) = (left_edge, columns);
            loop {
                last = self.scan(&mut at, top.wrapping_add(row), &mut left);
                units = units.wrapping_add(1);
                if last.1 == 0 {
                    break;
                }
                units = units.wrapping_add(1).wrapping_add(last.1);
                if !wide && (last.0 > 0xFF || last.1 > 0xFF) {
                    units = 1;
                    break 'rows;
                }
            }
            row += 1;
            rows_left -= 1;
            if rows_left < 0 {
                break;
            }
        }
        let size = units.wrapping_mul(self.pixel_size);

        if !wide {
            if last.0 > 0xFF {
                glyph.warnings |= 1;
            }
            if last.1 > 0xFF {
                glyph.warnings |= 2;
            }
            if glyph.warnings != 0 {
                // A glyph of one empty row in its place. When it is the blank end of
                // the last row that is too long, the original has already counted the
                // whole glyph's size and leaves the rest of it unwritten.
                if size != 1 {
                    return Err(Invalid(
                        "a glyph whose last row ends in more than 255 blank pixels",
                    ));
                }
                glyph.data = vec![0, 0, 0, 1, 0];
                return Ok(glyph);
            }
            glyph.data.reserve(4 + size as usize);
            glyph.data.push(left_cut as u8);
            if left_cut > 0xFF {
                glyph.warnings |= 4;
            }
            glyph.data.push(top_cut as u8);
            // The original tests the top cut three times over, not the two sizes.
            if top_cut > 0xFF {
                glyph.warnings |= 8 | 0x10 | 0x20;
            }
            glyph.data.push(cut_width as u8);
            glyph.data.push(cut_height as u8);
        } else {
            glyph.data.reserve(0xC + size as usize);
            glyph
                .data
                .extend(self.pixel_size.wrapping_neg().to_le_bytes());
            for value in [left_cut, top_cut, cut_width, cut_height] {
                glyph.data.extend((value as u16).to_le_bytes());
            }
        }

        let mut row = 0u32;
        let mut rows_left = cut_height.wrapping_sub(1) as i32;
        loop {
            let (mut at, mut left) = (left_edge, columns);
            loop {
                let (blank, drawn) = self.scan(&mut at, top.wrapping_add(row), &mut left);
                self.put_unit(&mut glyph.data, drawn);
                if drawn == 0 {
                    break;
                }
                self.put_unit(&mut glyph.data, blank);
                for n in 0..drawn {
                    let pixel = self.get(
                        at.wrapping_sub(drawn).wrapping_add(n),
                        top.wrapping_add(row),
                    );
                    self.put_pixel(&mut glyph.data, pixel);
                }
            }
            row += 1;
            rows_left -= 1;
            if rows_left < 0 {
                break;
            }
        }
        Ok(glyph)
    }

    /// Every glyph of the sheet, in the order found: by rows, then columns, of
    /// their first pixel met. The sheet is left all backdrop where glyphs were.
    pub fn glyphs(&mut self) -> Result<Vec<Glyph>, Invalid> {
        let mut found = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get(x, y) == self.backdrop {
                    continue;
                }
                // A rectangle that is grown until nothing drawn touches its edge from
                // inside: left and top are the last blank column and row before it,
                // right and bottom the first after it.
                let (mut left, mut top, mut right, mut bottom) = (x, y, x + 1, y + 1);
                loop {
                    let mut grown = false;
                    let mut row = top;
                    loop {
                        if self.is_drawn(left, row) {
                            left = left.wrapping_sub(1);
                            row = top;
                            grown = true;
                            continue;
                        }
                        row = row.wrapping_add(1);
                        if bottom < row {
                            break;
                        }
                    }
                    let mut column = left;
                    loop {
                        if self.is_drawn(column, top) {
                            top = top.wrapping_sub(1);
                            column = left;
                            grown = true;
                            continue;
                        }
                        column = column.wrapping_add(1);
                        if right < column {
                            break;
                        }
                    }
                    let mut row = bottom;
                    loop {
                        if self.is_drawn(right, row) {
                            right = right.wrapping_add(1);
                            row = bottom;
                            grown = true;
                            continue;
                        }
                        row = row.wrapping_sub(1);
                        if row == top {
                            break;
                        }
                    }
                    let mut column = right;
                    loop {
                        if self.is_drawn(column, bottom) {
                            bottom = bottom.wrapping_add(1);
                            column = right;
                            grown = true;
                            continue;
                        }
                        column = column.wrapping_sub(1);
                        if column == left {
                            break;
                        }
                    }
                    if !grown {
                        break;
                    }
                }
                let (mut cell_x, mut cell_y) = (left.wrapping_add(1), top.wrapping_add(1));
                let (mut width, mut height) =
                    (right.wrapping_sub(cell_x), bottom.wrapping_sub(cell_y));
                if self.soften {
                    if self.flags & DIRECT_COLOUR != 0 {
                        return Err(Invalid("an alpha made for a sheet of components"));
                    }
                    // One pixel more on every side; the original does not ask
                    // whether the sheet goes that far.
                    if cell_x == 0 || cell_y == 0 || right >= self.width || bottom >= self.height {
                        return Err(Invalid("a glyph at the sheet's edge to be softened"));
                    }
                    (cell_x, cell_y) = (cell_x - 1, cell_y - 1);
                    (width, height) = (width + 2, height + 2);
                    self.soften(cell_x, cell_y, width, height);
                }
                found.push(self.coded(cell_x, cell_y, width, height)?);
                for row in 0..height.max(1) {
                    for column in 0..width.max(1) {
                        self.erase(cell_x.wrapping_add(column), cell_y.wrapping_add(row));
                    }
                }
            }
        }
        Ok(found)
    }
}

/// The sheet's backdrop is what its four corners hold, if they agree; else 0.
pub fn backdrop_of(sheet: &Sheet) -> u32 {
    let corner = sheet.get(0, 0);
    let (right, bottom) = (sheet.width.wrapping_sub(1), sheet.height.wrapping_sub(1));
    if sheet.get(right, 0) == corner
        && sheet.get(right, bottom) == corner
        && sheet.get(0, bottom) == corner
    {
        corner
    } else {
        0
    }
}

/// The transparent colour: magenta, or in a palette the first entry that is
/// magenta, or the backdrop if none is.
pub fn transparent_of(flags: u32, backdrop: u32, palette: &[u32; 256]) -> u32 {
    const MAGENTA: u32 = 0xFF_00FF;
    if flags & DIRECT_COLOUR != 0 {
        return MAGENTA;
    }
    (0..256u32)
        .find(|&entry| palette[entry as usize] == MAGENTA)
        .unwrap_or(backdrop)
}

/// The bytes of a coded pixel for a sheet of direct colour: 2 if the format's masks fit 16
/// bits, else 4.
pub fn direct_pixel_size(format: &PixelFormat) -> u32 {
    if format.masks.iter().fold(0, |all, mask| all | mask) <= 0xFFFF {
        2
    } else {
        4
    }
}

/// A sheet's table: an offset for each glyph, then the records.
pub fn table_of(glyphs: &[Glyph]) -> Vec<u8> {
    let mut table = Vec::new();
    let mut offset = glyphs.len() as u32 * 4;
    for glyph in glyphs {
        table.extend(offset.to_le_bytes());
        offset = offset.wrapping_add(glyph.data.len() as u32);
    }
    for glyph in glyphs {
        table.extend(&glyph.data);
    }
    table
}

/// One table after another as one: the offsets of both, each moved by what
/// now lies before its records, then the records of both.
pub fn append(table: &mut Vec<u8>, more: &[u8]) -> Result<(), Invalid> {
    if more.is_empty() {
        return Ok(());
    }
    if table.is_empty() {
        table.extend(more);
        return Ok(());
    }
    let first = |bytes: &[u8]| u32::from_le_bytes(bytes[..4].try_into().unwrap());
    let index_of = |bytes: &[u8]| {
        let index = if bytes.len() < 4 {
            0
        } else {
            first(bytes) as usize
        };
        (index != 0 && index % 4 == 0 && index <= bytes.len())
            .then_some(index)
            .ok_or(Invalid(
                "a glyph table whose first offset is not its index's size",
            ))
    };
    let (own_index, more_index) = (index_of(table)?, index_of(more)?);
    let own_len = table.len() as u32;
    let mut joined = Vec::with_capacity(table.len() + more.len());
    for offset in table[..own_index].as_chunks::<4>().0 {
        joined.extend(first(offset).wrapping_add(more_index as u32).to_le_bytes());
    }
    for offset in more[..more_index].as_chunks::<4>().0 {
        joined.extend(first(offset).wrapping_add(own_len).to_le_bytes());
    }
    joined.extend(&table[own_index..]);
    joined.extend(&more[more_index..]);
    *table = joined;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Grid {
        width: u32,
        bytes: Vec<u8>,
    }

    impl SheetPixels for Grid {
        fn byte(&self, x: u32, y: u32) -> u8 {
            self.bytes[(y * self.width + x) as usize]
        }
        fn set_byte(&mut self, x: u32, y: u32, value: u8) {
            self.bytes[(y * self.width + x) as usize] = value;
        }
        fn dword(&self, _: u32, _: u32) -> u32 {
            unreachable!("a palette sheet")
        }
        fn set_dword(&mut self, _: u32, _: u32, _: u32) {
            unreachable!("a palette sheet")
        }
    }

    #[test]
    fn finds_and_codes() {
        // A 6 x 4 sheet of backdrop 9 with one glyph: 1 2 / 3 at (2, 1).
        let mut grid = Grid {
            width: 6,
            bytes: vec![9; 24],
        };
        grid.bytes[6 + 2] = 1;
        grid.bytes[6 + 3] = 2;
        grid.bytes[12 + 2] = 3;
        let mut convert = |p| p;
        let mut sheet = Sheet {
            flags: RUN_LENGTHS,
            pixel_size: 1,
            width: 6,
            height: 4,
            backdrop: 0,
            transparent: 0,
            convert: &mut convert,
            soften: false,
            pixels: &mut grid,
        };
        sheet.backdrop = backdrop_of(&sheet);
        assert_eq!(sheet.backdrop, 9);
        let mut palette = [0u32; 256];
        palette[5] = 0xFF_00FF;
        sheet.transparent = transparent_of(0, 9, &palette);
        assert_eq!(sheet.transparent, 5);
        let glyphs = sheet.glyphs().unwrap();
        assert_eq!(glyphs.len(), 1);
        assert_eq!((glyphs[0].x, glyphs[0].y), (2, 1));
        // Head: no cuts, 2 x 2; row 1: 2 pixels after no gap; row 2: 1 pixel.
        assert_eq!(glyphs[0].data, [0, 0, 2, 2, 2, 0, 1, 2, 0, 1, 0, 3, 0]);
        assert!(grid.bytes.iter().all(|&b| b == 9));
        let table = table_of(&glyphs);
        assert_eq!(&table[..4], &[4, 0, 0, 0]);
        let mut joined = table.clone();
        append(&mut joined, &table).unwrap();
        assert_eq!(&joined[..8], &[8, 0, 0, 0, 8 + 13, 0, 0, 0]);
    }
}
