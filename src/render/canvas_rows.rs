//! Which rows of the canvas the game has written since the last flush: the only rows the GPU
//! backend uploads and puts over the target.

use super::Target;
use super::gpu::UNWRITTEN;

/// Unwritten rows that may lie inside a run: a run costs an upload and a draw.
const RUN_GAP: usize = 16;

/// The canvas's rows written since the last flush, one flag a row, as the screen's pixels noted
/// them (every write through their handle marks its rows; writes in place mark them all): a
/// superset of the rows holding a written pixel. A row noted but holding none is uploaded
/// unwritten, which the overlay discards: the same picture. Pixels that do not note rows at the
/// canvas's pitch are looked at row by row. With `RC2K_CANVAS_CHECK` (a check) the rows are
/// looked at as well, and a row holding a written pixel but not noted is logged.
pub(super) fn written_rows(target: &Target, canvas: &[u8]) -> Vec<bool> {
    let stride = target.width * 4;
    let noted = (target.pitch == stride)
        .then(|| target.take_written_rows())
        .flatten()
        .filter(|rows| rows.len() == canvas.len() / stride.max(1));
    let Some(noted) = noted else {
        return rows_holding_written(canvas, stride);
    };
    if super::dev::switches().canvas_check {
        static LOGGED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let missed: Vec<usize> = rows_holding_written(canvas, stride)
            .iter()
            .zip(&noted)
            .enumerate()
            .filter(|(_, (held, noted))| **held && !**noted)
            .map(|(y, _)| y)
            .collect();
        if !missed.is_empty() && LOGGED.fetch_add(1, std::sync::atomic::Ordering::Relaxed) < 50 {
            crate::program_log::log(&format!(
                "render: canvas rows written but not noted: {missed:?}"
            ));
        }
    }
    noted
}

/// Each row of `stride` bytes: whether it holds a written pixel.
pub(super) fn rows_holding_written(canvas: &[u8], stride: usize) -> Vec<bool> {
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

/// The runs of written rows, each its first and last row: the panels at the top and the
/// bottom leave the rows between them to the 3D.
pub(super) fn written_runs(written: &[bool]) -> smallvec::SmallVec<[(usize, usize); 4]> {
    let mut runs = smallvec::SmallVec::<[(usize, usize); 4]>::new();
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
        assert_eq!(
            written_runs(&rows).as_slice(),
            [(3, 10), (40, 41), (99, 99)]
        );
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
