//! The error list loaded from the course's file: the file's 0x2000 bytes
//! (`race::level_init::load_error_list`); the developers' kstar mode keeps its recorded
//! places in it (`debug::kstar`).

/// The error list's size in bytes.
pub const ERROR_LIST_LEN: usize = 0x2000;

pub struct ErrorList {
    /// The error list read from the file named in the course (`race::load_error_list`).
    pub list: [u8; ERROR_LIST_LEN],
}

impl ErrorList {
    pub const fn new() -> Self {
        ErrorList {
            list: [0; ERROR_LIST_LEN],
        }
    }

    /// The little-endian dword at byte `offset` (any alignment); past the list ends the program.
    pub fn get_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(self.list[offset..offset + 4].try_into().unwrap())
    }

    pub fn set_u32(&mut self, offset: usize, value: u32) {
        self.list[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}
