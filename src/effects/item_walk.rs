//! The windscreen drops' and flakes' item tables share one way of taking a new item: a cursor
//! walks the items from where it stopped, at most `WINDOW` of them, to the first free one.

use crate::errors::{BadInput, Refused};

/// How many items one walk looks at.
pub const WINDOW: u32 = 32;

/// Where a walk ended.
pub struct Walk {
    /// The offset of the free item it found, if any.
    pub free: Option<u32>,
    /// The cursor after it: past the free item, or past the last one looked at.
    pub cursor: u32,
}

/// Walks from `cursor` over a table of `items` items of `item_len` bytes, wrapping at the end,
/// until `is_free` (given an item's offset) says yes. The cursor steps by whole items (only
/// its own writes move it); one off that grid is refused as `off_grid`, as it would read past
/// the items.
pub(super) fn walk_to_free(
    mut cursor: u32,
    items: u32,
    item_len: u32,
    is_free: impl Fn(u32) -> bool,
    off_grid: &'static str,
) -> Result<Walk, Refused> {
    let end = items * item_len;
    let step = |at: u32| {
        if at + item_len == end {
            0
        } else {
            at + item_len
        }
    };
    for _ in 0..WINDOW {
        if !cursor.is_multiple_of(item_len) || cursor >= end {
            return Err(BadInput(off_grid).into());
        }
        let found = cursor;
        cursor = step(cursor);
        if is_free(found) {
            return Ok(Walk {
                free: Some(found),
                cursor,
            });
        }
    }
    Ok(Walk { free: None, cursor })
}
