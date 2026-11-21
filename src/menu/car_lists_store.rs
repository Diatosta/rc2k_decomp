//! The lists of cars the menus walk: the championship's, the A8 game's, the single rally's, the
//! Clio's, the cars won, the scratch list, the arcade's and the shown cars. Each list is car
//! handles ended by -1. The saves and the menus write them; `MenuLists::car_entry` reads a list
//! on past its end into the lists that follow, in the order of the fields.

pub use crate::memory::session::CarEntries;
use crate::memory::session::{GameThread, Run};
use crate::menu::cars_store::listed_car_handle;
use std::cell::RefCell;

/// A row of `N` car handles ended by -1, plain dwords behind one `RefCell` (the game's thread
/// alone reads and writes them, each borrow lasting one statement).
pub struct CarRow<const N: usize>(GameThread<RefCell<[u32; N]>>);

impl<const N: usize> CarRow<N> {
    const fn new(entries: [u32; N]) -> Self {
        Self(GameThread::new(RefCell::new(entries)))
    }

    /// All of it.
    pub fn read(&self) -> [u32; N] {
        *self.0.get().borrow()
    }

    pub fn write(&self, entries: [u32; N]) {
        *self.0.get().borrow_mut() = entries;
    }
}

impl<const N: usize> CarEntries for CarRow<N> {
    fn room(&self) -> usize {
        N
    }
    fn entry(&self, n: usize) -> u32 {
        self.0.get().borrow()[n]
    }
    fn set_entry(&self, n: usize, value: u32) {
        self.0.get().borrow_mut()[n] = value;
    }
}

impl<const N: usize> Run for CarRow<N> {
    fn count(&self) -> usize {
        N
    }
    fn bits(&self, i: usize) -> u32 {
        self.entry(i)
    }
    fn set_bits(&self, i: usize, v: u32) {
        self.set_entry(i, v);
    }
}

/// The lists of cars.
pub struct CarLists {
    /// The championship's car list (`menu::event_flow`).
    pub cars_championship: CarRow<26>,
    /// The A8 rally championship's car list (45 entries ended by -1; `menu::remove_secret_cars`
    /// edits it with the first list).
    pub a8_championship_cars: CarRow<45>,
    /// The car list without the Clio, the first.
    pub cars_first: CarRow<34>,
    /// The car list with the Clio and the extra cars.
    pub cars_with_clio: CarRow<62>,
    /// The cars won.
    pub secret_cars: CarRow<12>,
    /// The scratch list `menu::remove_secret_cars` builds.
    pub cars_scratch: CarRow<50>,
    /// The second car list (arcade events).
    pub cars_second: CarRow<59>,
    /// The list of the cars shown (`menu::arcade_steps`).
    pub shown_cars: CarRow<48>,
}

/// The end mark in a row of car numbers: no car.
const END: usize = usize::MAX;

/// The starting car numbers of `cars_championship`.
#[rustfmt::skip]
const CARS_CHAMPIONSHIP: [usize; 26] = [
    17, 18, 19, 12, 11, 5, 9, 15, 16, 20, 1, 3, 8, 0, 13, 21, END, END, END, END, END, END, END,
    END, END, END,
];

/// The starting car numbers of `a8_championship_cars`.
#[rustfmt::skip]
const A8_CHAMPIONSHIP_CARS: [usize; 45] = [
    14, 4, 7, 2, 10, 22, END, END, END, END, END, END, END, END, END, 17, 7, END, 0, 1, 2, 3, 4, 5,
    6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, END, END, END, END,
];

/// The starting car numbers of `cars_first`.
#[rustfmt::skip]
const CARS_FIRST: [usize; 34] = [
    13, 21, 18, 19, 12, 11, 5, 9, 15, 16, 20, 1, 3, 8, 0, 17, 22, 10, 2, 7, 4, 14, END, END, END,
    END, END, END, END, END, END, END, END, END,
];

/// The starting car numbers of `cars_with_clio`.
#[rustfmt::skip]
const CARS_WITH_CLIO: [usize; 62] = [
    13, 21, 18, 19, 12, 11, 5, 9, 15, 16, 20, 1, 3, 8, 0, 17, 22, 10, 2, 7, 4, 14, 23, 24, 25, 26,
    27, 28, 29, 31, 32, END, END, END, END, END, END, END, END, END, END, END, END, END, END, END,
    END, END, END, END, END, END, END, END, END, END, END, END, END, END, END, END,
];

/// The starting car numbers of `secret_cars`.
#[rustfmt::skip]
const SECRET_CARS: [usize; 12] = [
    23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 6, END,
];

/// The starting car numbers of `cars_second`.
#[rustfmt::skip]
const CARS_SECOND: [usize; 59] = [
    13, 21, 18, 19, 12, 11, 5, 9, 15, 16, 20, 1, 3, 8, 0, 17, END, END, END, END, END, END, END,
    END, END, END, END, END, END, END, END, END, END, END, END, END, END, END, END, END, END, END,
    END, END, END, END, END, END, END, END, END, END, END, END, END, END, END, END, END,
];

/// The starting car numbers of `shown_cars`.
#[rustfmt::skip]
const SHOWN_CARS: [usize; 48] = [
    2, 4, 7, 10, 14, 22, END, END, END, END, END, END, END, END, END, END, END, END, END, END, END,
    END, END, END, END, END, END, END, END, END, END, END, END, END, END, END, END, END, END, END,
    END, END, END, END, END, END, END, END,
];

/// The handles of a row of car numbers, `END` as the end mark.
const fn car_row<const N: usize>(cars: [usize; N]) -> CarRow<N> {
    let mut entries = [u32::MAX; N];
    let mut n = 0;
    while n < N {
        if cars[n] != END {
            entries[n] = listed_car_handle(cars[n]);
        }
        n += 1;
    }
    CarRow::new(entries)
}

impl CarLists {
    pub const fn new() -> Self {
        CarLists {
            cars_championship: car_row(CARS_CHAMPIONSHIP),
            a8_championship_cars: car_row(A8_CHAMPIONSHIP_CARS),
            cars_first: car_row(CARS_FIRST),
            cars_with_clio: car_row(CARS_WITH_CLIO),
            secret_cars: car_row(SECRET_CARS),
            cars_scratch: CarRow::new([u32::MAX; 50]),
            cars_second: car_row(CARS_SECOND),
            shown_cars: car_row(SHOWN_CARS),
        }
    }
}

#[cfg(test)]
mod image_bytes_tests {
    use super::*;

    /// The record's whole bytes, as a checksum: a wrong dword in a row shows here.
    #[test]
    fn the_car_lists_keep_the_bytes_of_the_image() {
        let bytes: Vec<u8> = {
            let lists = CarLists::new();
            [
                &lists.cars_championship.read()[..],
                &lists.a8_championship_cars.read()[..],
                &lists.cars_first.read()[..],
                &lists.cars_with_clio.read()[..],
                &lists.secret_cars.read()[..],
                &lists.cars_scratch.read()[..],
                &lists.cars_second.read()[..],
                &lists.shown_cars.read()[..],
            ]
            .iter()
            .flat_map(|list| list.iter())
            .flat_map(|entry| entry.to_le_bytes())
            .collect()
        };
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in &bytes {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
        }
        assert_eq!(bytes.len(), 1344);
        assert_eq!(hash, 0x3775_c02f_200f_a292);
    }
}
