//! A cell that holds a name id (`rc2k_core::saves::name_id::NameId`) in the layout of the files
//! it is read from and written to (`var\hst.dat`, the race's rival tables): the cell is the file's
//! dword, and the program reads and writes the name id, never the number.

use crate::memory::session::PlainDword;
use rc2k_core::saves::name_id::NameId;

#[repr(transparent)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView, rc2k_derive::ZeroCells)]
pub struct NameIdCell(PlainDword);

impl NameIdCell {
    /// A cell holding the file's number `number`.
    pub const fn from_file(number: u32) -> Self {
        Self(PlainDword::new(number))
    }

    pub fn get(&self) -> NameId {
        NameId::from_file(self.0.get())
    }

    pub fn set(&self, name: NameId) {
        self.0.set(name.to_file());
    }

    /// The cell's file dword, for the code that moves the file's bytes in and out.
    pub fn file_dword(&self) -> &PlainDword {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cell keeps every file number through a name id and back.
    #[test]
    fn the_cell_keeps_the_files_numbers() {
        for number in [0, 0x5A6, 0x755784, 0x7557E4, 0x7557E5, u32::MAX] {
            let cell = NameIdCell::from_file(number);
            let name = cell.get();
            cell.set(name);
            assert_eq!(cell.0.get(), number);
        }
        let cell = NameIdCell::from_file(0);
        cell.set(NameId::of_player(1));
        assert_eq!(cell.0.get(), 0x7557A4);
        assert_eq!(cell.get(), NameId::of_player(1));
    }
}
