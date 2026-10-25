//! The 180 drivers' records (`unk_73B674`, 0x94 bytes each) as the functions that make up
//! default best times use them, and the table their names come from.

use crate::errors::BadInput;
use crate::strings::append_string_by_id::push_string;

const DRIVER_LEN: usize = 0x94;
pub const DRIVERS: u32 = 180;
/// More entries than the names' table can have.
const MOST_NAME_ENTRIES: u32 = 0x400;
/// The room a name has where these functions put it.
pub const NAME_ROOM: usize = 0x20;

/// A driver's record, by number.
pub struct Driver(usize);

impl Driver {
    pub fn number(number: u32) -> Result<Self, BadInput> {
        if number >= DRIVERS {
            return Err(BadInput("a driver past the 180"));
        }
        Ok(Self(
            crate::menu::drivers_store::DRIVERS.first() as usize + number as usize * DRIVER_LEN,
        ))
    }

    /// The dword at `offset` of the record.
    pub unsafe fn dword(&self, offset: usize) -> u32 {
        unsafe { (self.0 as *const u32).byte_add(offset).read() }
    }

    /// The id of the string that is the driver's name.
    pub unsafe fn name_id(&self) -> Result<u32, BadInput> {
        let entry = unsafe { self.dword(0) };
        if entry >= MOST_NAME_ENTRIES {
            return Err(BadInput("a driver whose name entry is far past the table"));
        }
        Ok(crate::menu::drivers_store::name_string(entry))
    }

    /// The name's bytes, without a NUL, as `strings::append_string_by_id` writes them.
    pub unsafe fn name(&self) -> Result<(u32, Vec<u8>), BadInput> {
        unsafe {
            let id = self.name_id()?;
            let mut name = Vec::new();
            push_string(&mut name, id);
            if name.len() > NAME_ROOM {
                return Err(BadInput("a driver's name longer than its place"));
            }
            Ok((id, name))
        }
    }
}

/// Stores a dword at `at + offset`.
pub unsafe fn put(at: u32, offset: u32, value: u32) {
    unsafe { ((at + offset) as *mut u32).write_unaligned(value) };
}

/// Copies bytes to `at + offset`.
pub unsafe fn put_bytes(at: u32, offset: u32, bytes: &[u8]) {
    unsafe { std::ptr::copy(bytes.as_ptr(), (at + offset) as *mut u8, bytes.len()) };
}
