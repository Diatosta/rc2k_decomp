//! Which rows of the canvas the game has written since the last flush: the only rows the GPU
//! backend uploads and puts over the target.

use super::Target;
use super::gpu::UNWRITTEN;

/// Unwritten rows that may lie inside a run: a run costs an upload and a draw.
const RUN_GAP: usize = 16;

/// The canvas's rows written since the last flush, one flag a row, as the screen's pixels noted
/// them (every write through their handle marks its rows; writes in place mark them all): a
/// superset of the rows holding a written pixel. Pixels that do not note rows at the canvas's
/// pitch are looked at row by row.
pub fn written_rows(target: &Target, canvas: &[u8]) -> Vec<bool> {
    let stride = target.width * 4;
    (target.pitch == stride)
        .then(|| target.take_written_rows())
        .flatten()
        .filter(|rows| rows.len() == canvas.len() / stride.max(1))
        .unwrap_or_else(|| rows_holding_written(canvas, stride))
}

/// Each row of `stride` bytes: whether it holds a written pixel.
pub fn rows_holding_written(canvas: &[u8], stride: usize) -> Vec<bool> {
    const BLOCK: usize = 256;
    let unwritten = UNWRITTEN.to_le_bytes();
    let blank: [u8; BLOCK] = std::array::from_fn(|n| unwritten[n % 4]);
    let row_written = |row: &[u8]| {
        let (blocks, rest) = row.as_chunks::<BLOCK>();
        blocks.iter().any(|block| *block != blank) || *rest != blank[..rest.len()]
    };
    canvas
        .chunks_exact(stride.max(1))
        .map(row_written)
        .collect()
}

/// The runs of written rows, each its first and last row.
pub fn written_runs(written: &[bool]) -> Vec<(usize, usize)> {
    let mut runs = Vec::<(usize, usize)>::new();
    for y in (0..written.len()).filter(|&y| written[y]) {
        match runs.last_mut() {
            Some((_, last)) if y - *last <= RUN_GAP => *last = y,
            _ => runs.push((y, y)),
        }
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_join_rows_a_short_gap_apart() {
        let mut rows = vec![false; 100];
        for y in [3, 4, 10, 40, 41, 99] {
            rows[y] = true;
        }
        assert_eq!(written_runs(&rows), [(3, 10), (40, 41), (99, 99)]);
        assert!(written_runs(&[false; 5]).is_empty());
    }

    #[test]
    fn rows_are_written_when_a_pixel_is_not_the_mark() {
        let mut canvas = vec![0u8; 3 * 8];
        canvas.as_chunks_mut::<4>().0.fill(UNWRITTEN.to_le_bytes());
        canvas[2 * 8 + 4] = 1;
        assert_eq!(rows_holding_written(&canvas, 8), [false, false, true]);
    }
}
