//! The lists of car numbers the options file holds: the numbers of the cars that can be chosen
//! in two lists (`saves::rebuild_car_lists` makes the car pointers of `menu::lists_store` from
//! them). Plain arrays behind one `RefCell`: the game's thread alone reads and writes them, each
//! borrow lasting one statement.

use crate::memory::session::GameThread;
use std::cell::{RefCell, RefMut};

/// How many entries each list of numbers has.
pub const LIST_LEN: usize = 40;

/// The lists of car numbers.
#[derive(Clone, Copy)]
pub struct NumberLists {
    /// The first list's car numbers, ended by -1 (the options file holds all 0x28 entries).
    pub first_numbers: [u32; LIST_LEN],
    /// The second list's car numbers, ended by -1 (the options file holds 0x14 of its entries,
    /// `saves::list_car_numbers` fills 0x28).
    pub second_numbers: [u32; LIST_LEN],
}

/// The lists of car numbers.
pub struct CarNumbers(GameThread<RefCell<NumberLists>>);

impl CarNumbers {
    pub const fn new() -> Self {
        CarNumbers(GameThread::new(RefCell::new(NumberLists {
            first_numbers: [0; LIST_LEN],
            second_numbers: [0; LIST_LEN],
        })))
    }

    /// A copy of the lists: no borrow outlives the statement that reads them.
    pub fn read(&self) -> NumberLists {
        *self.0.get().borrow()
    }

    /// The lists, to write.
    pub fn write(&self) -> RefMut<'_, NumberLists> {
        self.0.get().borrow_mut()
    }

    /// The first list as the options file holds it: 0x28 little-endian dwords.
    pub fn first_bytes(&self) -> Vec<u8> {
        le_bytes(&self.read().first_numbers)
    }

    /// Sets the first list from the file's bytes.
    pub fn set_first(&self, bytes: &[u8]) {
        set_from_le(&mut self.write().first_numbers, bytes);
    }

    /// The second list's first 0x14 entries as the options file holds them.
    pub fn second_bytes(&self) -> Vec<u8> {
        le_bytes(&self.read().second_numbers[..FILE_SECOND_LEN])
    }

    /// Sets the second list's first 0x14 entries from the file's bytes.
    pub fn set_second(&self, bytes: &[u8]) {
        set_from_le(&mut self.write().second_numbers[..FILE_SECOND_LEN], bytes);
    }
}

/// How many entries of the second list the options file holds.
pub const FILE_SECOND_LEN: usize = 20;

fn le_bytes(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}

fn set_from_le(words: &mut [u32], bytes: &[u8]) {
    for (word, chunk) in words.iter_mut().zip(bytes.as_chunks::<4>().0) {
        *word = u32::from_le_bytes(*chunk);
    }
}
