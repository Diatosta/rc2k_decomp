//! Rows of the debug pages read through the field they show, not through an IDA address: the
//! row's IDA address is only its label and identity.

use crate::session::{Dword, Float, PlainDword, PlainFloat};

/// A cell of the game's state as the pages read it.
pub trait Raw {
    /// Its 32 bits.
    fn raw(&self) -> u32;
}

impl Raw for Dword {
    fn raw(&self) -> u32 {
        self.get()
    }
}

impl Raw for Float {
    fn raw(&self) -> u32 {
        self.bits()
    }
}

impl Raw for PlainDword {
    fn raw(&self) -> u32 {
        self.get()
    }
}

impl Raw for crate::session::PlainInt {
    fn raw(&self) -> u32 {
        self.get() as u32
    }
}

impl Raw for PlainFloat {
    fn raw(&self) -> u32 {
        self.bits()
    }
}

impl Raw for crate::session::PlainLoose {
    fn raw(&self) -> u32 {
        self.get()
    }
}

/// The cell of field `$field` of car block `$block` (or of its element `$i`).
macro_rules! car_cell {
    ($block:literal, $field:ident) => {{
        fn cell<'a>(o: &$crate::owners::Owners<'a>) -> &'a dyn $crate::debug::row_cells::Raw {
            &o.race.car_blocks().cars[$block].$field
        }
        cell
    }};
    ($block:literal, $field:ident[$i:literal]) => {{
        fn cell<'a>(o: &$crate::owners::Owners<'a>) -> &'a dyn $crate::debug::row_cells::Raw {
            &o.race.car_blocks().cars[$block].$field[$i]
        }
        cell
    }};
}
pub(super) use car_cell;

/// The cell `$home` (a store of one cell), or its field at path `$field`.
macro_rules! store_cell {
    (|$o:ident| $home:expr) => {{
        fn cell<'a>($o: &$crate::owners::Owners<'a>) -> &'a dyn $crate::debug::row_cells::Raw {
            &$home
        }
        cell
    }};
    (|$o:ident| $home:expr, $($field:tt)+) => {{
        fn cell<'a>($o: &$crate::owners::Owners<'a>) -> &'a dyn $crate::debug::row_cells::Raw {
            &$home.$($field)+
        }
        cell
    }};
}
pub(super) use store_cell;

/// The cell at path `$field` (with indices) of the store `$home`.
macro_rules! indexed_cell {
    (|$o:ident| $home:expr, $($field:tt)+) => {{
        fn cell<'a>($o: &$crate::owners::Owners<'a>) -> &'a dyn $crate::debug::row_cells::Raw {
            &$home.$($field)+
        }
        cell
    }};
}
pub(super) use indexed_cell;
