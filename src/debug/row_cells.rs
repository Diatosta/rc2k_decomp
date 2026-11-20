//! Rows of the debug pages read through the field they show; the row's address is only its
//! label and identity.

use crate::memory::session::{Dword, Float, PlainDword, PlainFloat};

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

impl Raw for crate::memory::session::PlainInt {
    fn raw(&self) -> u32 {
        self.get() as u32
    }
}

impl Raw for PlainFloat {
    fn raw(&self) -> u32 {
        self.bits()
    }
}

impl Raw for u32 {
    fn raw(&self) -> u32 {
        *self
    }
}

impl Raw for i32 {
    fn raw(&self) -> u32 {
        *self as u32
    }
}

impl Raw for f32 {
    fn raw(&self) -> u32 {
        self.to_bits()
    }
}

/// A figure worked out for the pages (`debug::frame_figures`).
impl Raw for std::sync::atomic::AtomicU32 {
    fn raw(&self) -> u32 {
        self.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// The cell of field `$field` of car block `$block` (or of its element `$i`).
macro_rules! car_cell {
    ($block:literal, $field:ident) => {{
        fn cell<'a>(o: &$crate::owners::Owners<'a>) -> u32 {
            $crate::debug::row_cells::Raw::raw(&o.race.car_blocks().cars[$block].$field)
        }
        cell
    }};
    ($block:literal, $field:ident[$i:literal]) => {{
        fn cell<'a>(o: &$crate::owners::Owners<'a>) -> u32 {
            $crate::debug::row_cells::Raw::raw(&o.race.car_blocks().cars[$block].$field[$i])
        }
        cell
    }};
}
pub(super) use car_cell;

/// The cell `$home` (a store of one cell), or its field at path `$field`.
macro_rules! store_cell {
    (|$o:ident| $home:expr) => {{
        fn cell<'a>($o: &$crate::owners::Owners<'a>) -> u32 {
            $crate::debug::row_cells::Raw::raw(&$home)
        }
        cell
    }};
    (|$o:ident| $home:expr, $($field:tt)+) => {{
        fn cell<'a>($o: &$crate::owners::Owners<'a>) -> u32 {
            $crate::debug::row_cells::Raw::raw(&$home.$($field)+)
        }
        cell
    }};
}
pub(super) use store_cell;

/// The cell at path `$field` (with indices) of the store `$home`.
macro_rules! indexed_cell {
    (|$o:ident| $home:expr, $($field:tt)+) => {{
        fn cell<'a>($o: &$crate::owners::Owners<'a>) -> u32 {
            $crate::debug::row_cells::Raw::raw(&$home.$($field)+)
        }
        cell
    }};
}
pub(super) use indexed_cell;
