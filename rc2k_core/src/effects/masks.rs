//! The marks' masks: the points of faces gathered into lists, shapes' rows of spans filled into a
//! mask of 0x8000 bytes (rows of 0x100) with the shape's number, 0xFF elsewhere, and a mask put
//! into the marks' texture when the settings switch masks.

use crate::Invalid;
use crate::effects::points::{Collected, add_points};

/// A mask's bytes.
pub const MASK_LEN: usize = 0x8000;
/// A row's bytes.
pub const ROW: u32 = 0x100;
/// The most points a list holds.
pub const MOST_POINTS: u32 = 0x40;
/// The mask's byte where no shape is.
pub const EMPTY: u8 = 0xFF;

/// A list of points: their ids and the face each came with.
pub struct PointList<'a> {
    pub count: u32,
    pub ids: &'a mut [u32],
    pub faces: &'a mut [u32],
}

impl PointList<'_> {
    /// Refuses a list of more than 0x40.
    pub fn check(&self) -> Result<(), Invalid> {
        if self.count > MOST_POINTS {
            return Err(Invalid("a points list of more than 0x40"));
        }
        Ok(())
    }

    /// `collect_points_a` / `_b`: face `face`'s points (`points` of them, signed; `point(k)`
    /// asked for each looked at) added unless there already, up to 0x40, each with the
    /// face. Once the list is full the rest are not looked at.
    pub fn add_face(
        &mut self,
        face: u32,
        points: u32,
        point: impl Fn(u32) -> u32,
    ) -> Result<Collected, Invalid> {
        self.check()?;
        let most = MOST_POINTS as usize;
        let faces = &mut *self.faces;
        Ok(add_points(
            &mut self.count,
            &mut self.ids[..most],
            points,
            point,
            |n, _| faces[n] = face,
        ))
    }
}

/// The masks' cursor over the work block: where the next run goes and how many there are.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MaskCursor {
    pub cursor: u32,
    pub runs: u32,
    pub run_count: u32,
}

impl MaskCursor {
    /// `reset_mask_cursor`: the cursor at the work block, the runs at their start, none.
    pub fn reset(work: u32, runs_start: u32) -> Self {
        Self {
            cursor: work,
            runs: runs_start,
            run_count: 0,
        }
    }
}

/// A span of a row: where it starts and how many bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub x: u32,
    pub len: u32,
}

/// One entry of a shape's rows: its first row and a span a row from there.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShapeRows {
    pub first_row: u32,
    pub spans: Vec<Span>,
}

impl ShapeRows {
    /// Whether every span is inside the mask (signed).
    pub fn fit(&self) -> Result<(), Invalid> {
        for (i, s) in self.spans.iter().enumerate() {
            let row = self.first_row.wrapping_add(i as u32) as i32 as i64;
            let at = s.x as i32 as i64 + row * ROW as i64;
            if at < 0 || at as u64 + s.len as u64 > MASK_LEN as u64 {
                return Err(Invalid("a mask span outside the mask"));
            }
        }
        Ok(())
    }

    /// `fill_mask_spans`: each span filled with `value`. Refuses a span outside the mask,
    /// before writing.
    pub fn fill(&self, mask: &mut [u8], value: u8) -> Result<(), Invalid> {
        self.fit()?;
        for (i, s) in self.spans.iter().enumerate() {
            let at =
                s.x.wrapping_add(self.first_row.wrapping_add(i as u32).wrapping_mul(ROW)) as usize;
            mask[at..at + s.len as usize].fill(value);
        }
        Ok(())
    }
}

/// `draw_mask_shape`: shape `shape`'s two entries (2 shape, 2 shape + 1) filled with its
/// number's low byte. Refuses a span outside the mask, before writing.
pub fn draw_shape(mask: &mut [u8], entries: [&ShapeRows; 2], shape: u32) -> Result<(), Invalid> {
    for e in entries {
        e.fit()?;
    }
    for e in entries {
        e.fill(mask, shape as u8)?;
    }
    Ok(())
}

/// `draw_mask`: the mask all `EMPTY`, then each shape drawn (two entries a shape). Refuses no
/// shapes (the loop would run round) or a span outside the mask, before writing.
pub fn draw_mask(mask: &mut [u8], entries: &[ShapeRows]) -> Result<(), Invalid> {
    if entries.len() < 2 {
        return Err(Invalid("a mask of no shapes: the loop would run round"));
    }
    for e in entries {
        e.fit()?;
    }
    mask[..MASK_LEN].fill(EMPTY);
    for (k, pair) in entries.as_chunks::<2>().0.iter().enumerate() {
        draw_shape(mask, [&pair[0], &pair[1]], k as u32)?;
    }
    Ok(())
}

/// The rows of a mask put into the texture, and the texture's row the first goes to.
pub(crate) const TEXTURE_ROWS: u32 = 0x7E;
pub const TEXTURE_ROW_AT: u32 = 0x80;

/// `swap_mask`: with the settings' switch not the one last seen, which mask goes into the
/// texture: the second with the switch set; None when it has not changed.
pub fn swapped(switch: u32, seen: u32) -> Option<bool> {
    (switch != seen).then_some(switch != 0)
}

/// Refuses a locked texture wider than a mask's row (its rows would be read past them).
pub fn check_width(width: u32) -> Result<(), Invalid> {
    if width > ROW {
        return Err(Invalid("a locked mask texture wider than the mask's 0x100"));
    }
    Ok(())
}

/// The pixels put, in order: (row, column, the mask's dword); each row's first pixel put whatever
/// the width.
pub fn texture_pixels(width: u32) -> impl Iterator<Item = (u32, u32, u32)> {
    (0..TEXTURE_ROWS)
        .flat_map(move |row| (0..width.max(1)).map(move |col| (row, col, (row << 8) + col)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points() {
        let (mut ids, mut faces) = ([0u32; 0x40], [0u32; 0x40]);
        ids[0] = 7;
        let mut list = PointList {
            count: 1,
            ids: &mut ids,
            faces: &mut faces,
        };
        let pts = [7u32, 9, 9, 3];
        let c = list.add_face(0x55, 4, |k| pts[k as usize]).unwrap();
        assert_eq!(
            c,
            Collected {
                last_point: Some(3),
                stopped: Some(2),
                stepped: 4
            }
        );
        assert_eq!(list.count, 3);
        assert_eq!(
            (&ids[..3], &faces[..3]),
            (&[7, 9, 3][..], &[0, 0x55, 0x55][..])
        );
        let mut list = PointList {
            count: 0x40,
            ids: &mut ids,
            faces: &mut faces,
        };
        assert_eq!(
            list.add_face(1, 5, |_| unreachable!()).unwrap(),
            Collected::default()
        );
        assert_eq!(
            list.add_face(1, u32::MAX, |_| 0).unwrap(),
            Collected::default()
        );
        list.count = 0x41;
        assert!(list.add_face(1, 1, |_| 0).is_err());
    }

    #[test]
    fn masks() {
        let mut mask = vec![0u8; MASK_LEN];
        let rows = |first_row, spans: &[(u32, u32)]| ShapeRows {
            first_row,
            spans: spans.iter().map(|&(x, len)| Span { x, len }).collect(),
        };
        let entries = [
            rows(1, &[(2, 3), (0, 1)]),
            rows(0x7F, &[(0xFE, 2)]),
            rows(0, &[]),
            rows(5, &[(0, 0x100)]),
        ];
        draw_mask(&mut mask, &entries).unwrap();
        assert_eq!(&mask[0x100..0x106], &[0xFF, 0xFF, 0, 0, 0, 0xFF]);
        assert_eq!(mask[0x200], 0);
        assert_eq!(&mask[0x7FFD..], &[0xFF, 0, 0]);
        assert!(mask[0x500..0x600].iter().all(|&b| b == 1));
        assert!(draw_mask(&mut mask, &entries[..1]).is_err());
        let outside = [rows(0x80, &[(0, 1)]), rows(0, &[])];
        mask.fill(3);
        assert!(draw_mask(&mut mask, &outside).is_err());
        assert!(mask.iter().all(|&b| b == 3));
        assert!(rows(u32::MAX, &[(0x100, 1)]).fit().is_ok());
        assert!(rows(u32::MAX, &[(0xFF, 1)]).fit().is_err());
        assert_eq!(
            MaskCursor::reset(5, 9),
            MaskCursor {
                cursor: 5,
                runs: 9,
                run_count: 0
            }
        );
    }

    #[test]
    fn swaps() {
        assert_eq!(swapped(1, 1), None);
        assert_eq!(swapped(2, 0), Some(true));
        assert_eq!(swapped(0, 2), Some(false));
        assert!(check_width(0x101).is_err() && check_width(0x100).is_ok());
        let p: Vec<_> = texture_pixels(0).collect();
        assert_eq!((p.len(), p[1]), (0x7E, (1, 0, 0x100)));
        let p: Vec<_> = texture_pixels(3).collect();
        assert_eq!((p.len(), p[4]), (3 * 0x7E, (1, 1, 0x101)));
    }
}
