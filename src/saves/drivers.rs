//! The 180 drivers' records (`g_drivers`, 0x94 bytes each) as the functions that make up
//! default best times use them, and the table their names come from.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::menu::owner::menus;
use crate::strings::append_string_by_id::push_string;

pub const DRIVERS: u32 = 180;
/// More entries than the names' table can have.
const MOST_NAME_ENTRIES: u32 = 0x400;
/// The room a name has where these functions put it.
pub const NAME_ROOM: usize = 0x20;

/// A driver's record, by number.
pub struct Driver(&'static crate::menu::drivers_store::Driver);

impl Driver {
    pub fn number(number: u32) -> Result<Self, BadInput> {
        if number >= DRIVERS {
            return Err(BadInput("a driver past the 180"));
        }
        Ok(Self(&menus().cars.drivers.drivers[number as usize]))
    }

    /// The dword at `offset` of the record.
    /// +8: the class.
    pub fn class(&self) -> u32 {
        self.0.class.get()
    }

    /// +0x14.
    pub fn nation(&self) -> u32 {
        self.0.nation.get()
    }

    /// +0x18.
    pub fn car(&self) -> u32 {
        self.0.car.get()
    }

    /// The id of the string that is the driver's name.
    pub fn name_id(&self, menus: &Menus) -> Result<u32, BadInput> {
        let entry = self.0.name_entry.get();
        if entry >= MOST_NAME_ENTRIES {
            return Err(BadInput("a driver whose name entry is far past the table"));
        }
        crate::menu::drivers_store::name_string(menus, entry)
    }

    /// The name's bytes, without a NUL, as `strings::append_string_by_id` writes them.
    pub fn name(&self, menus: &Menus) -> Result<(u32, Vec<u8>), BadInput> {
        let id = self.name_id(menus)?;
        let mut name = Vec::new();
        push_string(&mut name, id)?;
        if name.len() > NAME_ROOM {
            return Err(BadInput("a driver's name longer than its place"));
        }
        Ok((id, name))
    }
}
