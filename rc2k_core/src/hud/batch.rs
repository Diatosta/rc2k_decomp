//! The HUD's batch of vertices and indices: polygons gathered with one texture and drawn by the
//! device in one go; a change of texture or a batch fuller than 0x64 vertices draws what is there
//! first.

use crate::Invalid;

/// The batch's room: vertices (0x20 bytes each) and indices (words).
pub(crate) const VERTEX_ROOM: u32 = 0x400;
pub(crate) const INDEX_ROOM: u32 = 0x600;
/// More vertices than this and the batch is drawn before the next polygon.
pub const FULL: u32 = 0x64;
/// The most vertices of a polygon the batch takes.
pub const MOST_VERTICES: u32 = 0x10;
/// The batch's texture when it has none.
pub const NO_TEXTURE: u32 = u32::MAX;

/// The HUD's layouts, by the record's layout number (`draw_hud`): 1 and 2 the two-player split ones, 3
/// and 4 the larger split screens', anything else the single player's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Single,
    One,
    Two,
    /// Three or four players, each their part of the screen.
    Many(u32),
}

impl Layout {
    pub fn of(field: u32) -> Layout {
        match field {
            1 => Layout::One,
            2 => Layout::Two,
            3 | 4 => Layout::Many(field),
            _ => Layout::Single,
        }
    }
}

/// `flush_batch`: whether the batch is drawn (and emptied): it holds indices.
pub fn flush_due(indices: u32) -> bool {
    indices != 0
}

/// `add_polygon`'s first test: the batch is drawn first when it holds more than `FULL`
/// vertices or its texture is not the one asked for.
pub fn flushes(vertices: u32, texture: u32, batch_texture: u32) -> bool {
    vertices > FULL || texture != batch_texture
}

/// Refuses a polygon of no vertices or more than `MOST_VERTICES`, and one the batch has no room for
/// (counts as the flush leaves them: emptied when it draws).
pub fn check_polygon(
    count: u32,
    vertices: u32,
    indices: u32,
    flushes: bool,
) -> Result<(), Invalid> {
    if count == 0 || count > MOST_VERTICES {
        return Err(Invalid("a HUD polygon of 0 or too many vertices"));
    }
    let [vc, ic] = if flushes && flush_due(indices) {
        [0, 0]
    } else {
        [vertices, indices]
    };
    if vc + count > VERTEX_ROOM || ic + 6 > INDEX_ROOM {
        return Err(Invalid("a HUD polygon past the batch"));
    }
    Ok(())
}

/// The indices a polygon of `count` vertices from vertex `first` adds: a triangle, and with
/// other than three vertices a second one making a quad (0, 2, 3).
pub fn polygon_indices(first: u32, count: u32) -> Vec<u16> {
    let mut out: Vec<u16> = (0..3).map(|k| (first + k) as u16).collect();
    if count != 3 {
        out.extend([0, 2, 3].map(|d| (first + d) as u16));
    }
    out
}

/// The kinds' first slots (0x400 entries; the HUD asks for 0xC0 ..
/// 0xDC).
pub(crate) const KIND_SLOTS: u32 = 0x400;

/// `set_batch_texture`: the entry of the texture shown; refuses one
/// past the table, before anything.
pub fn texture_slot(shown: u32) -> Result<usize, Invalid> {
    if shown >= KIND_SLOTS {
        return Err(Invalid("a HUD texture past g_kindFirstSlots"));
    }
    Ok(shown as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch() {
        assert_eq!(
            [0, 1, 2, 3, 4, 5].map(Layout::of),
            [
                Layout::Single,
                Layout::One,
                Layout::Two,
                Layout::Many(3),
                Layout::Many(4),
                Layout::Single
            ]
        );
        assert!(flushes(0x65, 1, 1) && flushes(0, 1, 2) && !flushes(0x64, 1, 1));
        assert!(check_polygon(0, 0, 0, false).is_err());
        assert!(check_polygon(0x11, 0, 0, false).is_err());
        assert!(check_polygon(4, 0x3FD, 0, false).is_err());
        assert!(check_polygon(4, 0x3FD, 6, true).is_ok());
        assert!(check_polygon(4, 0x3FD, 0, true).is_err());
        assert_eq!(polygon_indices(8, 3), [8, 9, 10]);
        assert_eq!(polygon_indices(8, 4), [8, 9, 10, 8, 10, 11]);
        assert!(texture_slot(0x400).is_err());
        assert_eq!(texture_slot(0xC0), Ok(0xC0));
    }
}
