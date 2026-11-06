//! The menu module's owner (step 11, owned state): the menus' typed stores in one struct,
//! grouped by what they are for, in place of global stores. Reached through `menus()`, the
//! boundary for callers that do not take a group as a parameter yet.

use crate::menu::build_notice::BuildNotice;
use crate::menu::car_numbers_store::CarNumbers;
use crate::menu::cars_store::CarTable;
use crate::menu::drivers_store::DriverTable;
use crate::menu::item_sprites_store::ItemSprites;
use crate::menu::keys_store::KeyTables;
use crate::menu::play_films::Films;

/// The cars, their numbers and the drivers.
pub struct Cars {
    pub table: CarTable,
    pub numbers: CarNumbers,
    pub drivers: DriverTable,
}

impl Cars {
    pub const fn new() -> Self {
        Cars {
            table: CarTable::new(),
            numbers: CarNumbers::new(),
            drivers: DriverTable::new(),
        }
    }
}

/// The fixed pieces the menus show: item sprites, film names and the build notice.
pub struct Panels {
    pub item_sprites: ItemSprites,
    pub films: Films,
    pub notice: BuildNotice,
}

impl Panels {
    pub const fn new() -> Self {
        Panels {
            item_sprites: ItemSprites::new(),
            films: Films::new(),
            notice: BuildNotice::new(),
        }
    }
}

/// The menu module's state.
pub struct Menus {
    pub cars: Cars,
    pub panels: Panels,
    pub keys: Keys,
}

/// The keyboard tables.
pub struct Keys {
    pub tables: KeyTables,
}

impl Keys {
    pub const fn new() -> Self {
        Keys {
            tables: KeyTables::new(),
        }
    }
}

/// The one instance. Reached through `menus()`; `moved` registers its stores' places.
pub static MENUS: Menus = Menus {
    cars: Cars::new(),
    panels: Panels::new(),
    keys: Keys::new(),
};

/// The menus' state, for a caller that does not take it as a parameter yet.
pub fn menus() -> &'static Menus {
    &MENUS
}
