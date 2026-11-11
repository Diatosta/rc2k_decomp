//! Rows of the debug pages read through the field they show, not through an IDA address: the
//! field is checked at compile time against the address the row's tooltip names. The rows are
//! rewritten to these by `tools/debug_pages/typed_rows.py`.

use crate::session::{Dword, Float};

/// A cell of the game's state as the pages read it.
pub trait Raw: Sync {
    /// Its 32 bits.
    fn raw(&self) -> u32;
    /// Its game address.
    #[cfg(test)]
    fn address(&self) -> u32;
}

impl Raw for Dword {
    fn raw(&self) -> u32 {
        self.get()
    }

    #[cfg(test)]
    fn address(&self) -> u32 {
        crate::pointers::game_address(std::ptr::from_ref(self))
    }
}

impl Raw for Float {
    fn raw(&self) -> u32 {
        self.bits()
    }

    #[cfg(test)]
    fn address(&self) -> u32 {
        crate::pointers::game_address(std::ptr::from_ref(self))
    }
}

/// The cell of field `$field` of car block `$block` (or of its element `$i`), whose IDA
/// address is `$at`: a compile error when the field is not there.
macro_rules! car_cell {
    ($block:literal, $field:ident, $at:literal) => {{
        const _: () = assert!(
            crate::race::blocks_store::START
                + $block * crate::race::blocks_store::BLOCK_LEN
                + core::mem::offset_of!(crate::physics::car_store::Car, $field)
                == $at
        );
        fn cell() -> &'static dyn $crate::debug::row_cells::Raw {
            &crate::race::blocks_store::CAR_BLOCKS.cars[$block].$field
        }
        cell
    }};
    ($block:literal, $field:ident[$i:literal], $at:literal) => {{
        const _: () = assert!(
            crate::race::blocks_store::START
                + $block * crate::race::blocks_store::BLOCK_LEN
                + core::mem::offset_of!(crate::physics::car_store::Car, $field)
                + $i * 4
                == $at
        );
        fn cell() -> &'static dyn $crate::debug::row_cells::Raw {
            &crate::race::blocks_store::CAR_BLOCKS.cars[$block].$field[$i]
        }
        cell
    }};
}
pub(super) use car_cell;

/// The cell `$home` (a store of one cell) whose IDA address is `$at`; `$ida` is where the
/// store's moved range starts.
macro_rules! store_cell {
    ($ida:expr, $home:expr, $at:literal) => {{
        const _: () = assert!($ida == $at);
        fn cell() -> &'static dyn $crate::debug::row_cells::Raw {
            &$home
        }
        cell
    }};
    // A path without indices into a record of type `$ty`: checked at compile time.
    ($ida:expr, $home:expr, $ty:ty, $at:literal, $($field:tt)+) => {{
        const _: () = assert!($ida + core::mem::offset_of!($ty, $($field)+) == $at);
        fn cell() -> &'static dyn $crate::debug::row_cells::Raw {
            &$home.$($field)+
        }
        cell
    }};
}
pub(super) use store_cell;

/// The cell at path `$field` (with indices) of the store `$home`: its address is checked
/// against `$at` by `row_paths::typed_rows_are_where_their_addresses_say`.
macro_rules! indexed_cell {
    ($home:expr, $($field:tt)+) => {{
        fn cell() -> &'static dyn $crate::debug::row_cells::Raw {
            &$home.$($field)+
        }
        cell
    }};
}
pub(super) use indexed_cell;
