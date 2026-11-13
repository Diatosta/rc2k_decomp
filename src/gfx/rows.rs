//! A locked surface's or texture's pixels by row: where the first row starts, the bytes from one
//! row to the next (the pitch, the gap after a row's pixels included) and how many rows there are.
//! Every row is reached from one place bounded to the rows' extent (pitch x rows), so a row past
//! the last, or bytes running past the last row, are refused.
//! Row y starts at first + y x pitch, and a drawer that works out its pitch its own way
//! (`row_dwords` x 4 + the gap) passes that pitch.

use crate::errors::BadInput;
use crate::gfx::locked_pixels::Place;

/// The rows of a locked surface or texture.
#[derive(Clone, Debug)]
pub struct Rows {
    /// The first row's first byte, limited to the rows' extent.
    first: Place,
    pitch: u32,
    rows: u32,
}

impl Rows {
    /// `rows` rows of `pitch` bytes in the pixels `handle` names (`gfx::locked_pixels`).
    pub fn new(handle: u32, pitch: u32, rows: u32) -> Self {
        Self {
            first: Place::start(handle).limited(pitch.wrapping_mul(rows)),
            pitch,
            rows,
        }
    }

    /// The drawing surface's (`DRAW_SURFACE`: its pixels, pitch and height); refused while the
    /// picture buffer is the surface (`gfx::surface::handle`).
    pub fn surface(gfx: &crate::gfx::owner::Gfx) -> Result<Self, BadInput> {
        let drawing = &gfx.drawing;
        Ok(Self::new(
            crate::gfx::surface::handle(gfx)?,
            drawing.draw_surface.pitch.get(),
            drawing.draw_surface.height.get(),
        ))
    }

    /// Whether there are no pixels (a null first row).
    pub fn is_null(&self) -> bool {
        self.first.is_null()
    }

    pub const fn count(&self) -> u32 {
        self.rows
    }

    /// Row `y`'s byte `at`; refused for a row past the last.
    pub fn at(&self, y: u32, at: u32) -> Result<Place, BadInput> {
        if y >= self.rows {
            return Err(BadInput("a row past the surface's last"));
        }
        Ok(self
            .first
            .offset(y.wrapping_mul(self.pitch).wrapping_add(at)))
    }

    /// The byte `offset` from the first row's start (a drawer's plan of byte places), bounded to
    /// the rows' extent as any row's.
    pub fn byte(&self, offset: u32) -> Place {
        self.first.offset(offset)
    }

    pub fn read(&self, y: u32, at: u32, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
        self.at(y, at)?.read_bytes(len, what)
    }

    pub fn write(&self, y: u32, at: u32, bytes: &[u8], what: &str) -> Result<(), BadInput> {
        self.at(y, at)?.write_bytes(bytes, what)
    }

    pub fn fill(&self, y: u32, at: u32, value: u8, len: usize, what: &str) -> Result<(), BadInput> {
        self.at(y, at)?.fill(value, len, what)
    }
}

/// `rows` rows of the drawing surface (a locked texture) cleared from byte `start` of its first
/// (each row's dwords, then the row's gap).
pub fn clear_surface_rows(
    start: u32,
    rows: u32,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), BadInput> {
    let drawing = &gfx.drawing;
    let dwords = drawing.draw_surface.row_dwords.get();
    let surface = Rows::new(
        crate::gfx::surface::handle(gfx)?,
        (4 * dwords).wrapping_add(drawing.draw_surface.row_gap.get()),
        drawing.draw_surface.height.get(),
    );
    for y in 0..rows {
        surface.fill(y, start, 0, 4 * dwords as usize, "the surface's rows")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::locked_pixels as locked;
    use std::sync::Arc;

    #[test]
    fn rows_are_reached_by_pitch_and_bounded_to_their_extent() {
        let memory = Arc::new(crate::pool::PoolBytes::from_vec(vec![0xAAu8; 4 * 8]));
        let handle = 0x7FFF_0001;
        locked::register(handle, &memory);
        // Three rows of 8 bytes, the last 8 of the memory not theirs.
        let rows = Rows::new(handle, 8, 3);
        rows.write(1, 2, &[1, 2, 3], "test").unwrap();
        assert_eq!(rows.read(1, 2, 3, "test").unwrap(), [1, 2, 3]);
        rows.fill(2, 0, 7, 8, "test").unwrap();
        assert_eq!(rows.read(2, 0, 8, "test").unwrap(), [7; 8]);
        assert!(rows.write(3, 0, &[1], "test").is_err(), "a fourth row");
        assert!(
            rows.write(2, 4, &[0; 5], "test").is_err(),
            "past the last row"
        );
        rows.byte(9).write_bytes(&[5], "test").unwrap();
        assert!(
            rows.byte(24).write_bytes(&[5], "test").is_err(),
            "a byte past the rows"
        );
        let bytes = memory.to_vec();
        assert_eq!(bytes[24..], [0xAA; 8]);
        assert_eq!(bytes[9..13], [5, 1, 2, 3]);
        drop(rows);
        drop(memory);
        assert!(locked::bytes_of(handle).is_none(), "gone bytes are refused");
    }
}
