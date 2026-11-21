//! The players' names and the text being typed: the four players' names (0x20 bytes each) and
//! what follows them, the name `menu::standings` keeps, and the text as it was before an entry
//! started. Each area is plain bytes behind one `RefCell` (the game's thread alone reads and
//! writes them, each borrow lasting one statement).

use crate::errors::BadInput;
use crate::memory::place::{BytePlace, Run};
use crate::memory::session::{ByteBuffer, GameThread};
use std::cell::RefCell;

/// A run of `N` plain bytes: a name area.
pub struct NameArea<const N: usize>(GameThread<RefCell<[u8; N]>>);

impl<const N: usize> NameArea<N> {
    pub const fn new() -> Self {
        Self(GameThread::new(RefCell::new([0; N])))
    }

    pub fn get(&self, index: usize) -> u8 {
        self.0.get().borrow()[index]
    }

    pub fn set(&self, index: usize, value: u8) {
        self.0.get().borrow_mut()[index] = value;
    }

    /// All of it.
    pub fn read(&self) -> [u8; N] {
        *self.0.get().borrow()
    }

    pub fn write(&self, value: &[u8; N]) {
        *self.0.get().borrow_mut() = *value;
    }

    /// `len` bytes from `offset` (`what` names them in a refusal past the end).
    pub fn read_at(&self, offset: usize, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
        let end = Self::end(offset, len, what)?;
        Ok(self.0.get().borrow()[offset..end].to_vec())
    }

    /// `data` written from `offset`.
    pub fn write_at(&self, offset: usize, data: &[u8], what: &str) -> Result<(), BadInput> {
        let end = Self::end(offset, data.len(), what)?;
        self.0.get().borrow_mut()[offset..end].copy_from_slice(data);
        Ok(())
    }

    /// Where `len` bytes from `offset` end, refused past the area.
    fn end(offset: usize, len: usize, what: &str) -> Result<usize, BadInput> {
        match offset.checked_add(len) {
            Some(end) if end <= N => Ok(end),
            _ => {
                crate::memory::host_hooks::log(&format!(
                    "names: {what}: {len} bytes at {offset:#x} of a {N:#x}-byte buffer"
                ));
                Err(BadInput("past the end of a buffer"))
            }
        }
    }
}

impl<const N: usize> Run for NameArea<N> {
    fn len(&self) -> usize {
        N
    }
    fn get(&self, n: usize) -> u8 {
        NameArea::get(self, n)
    }
    fn set(&self, n: usize, v: u8) {
        NameArea::set(self, n, v);
    }
}

impl<const N: usize> ByteBuffer for NameArea<N> {
    fn read_vec(&self) -> Vec<u8> {
        self.read().to_vec()
    }
    fn get_byte(&self, index: usize) -> u8 {
        self.get(index)
    }
    fn get_u32_at(&self, offset: usize) -> u32 {
        let area = self.0.get().borrow();
        u32::from_le_bytes(std::array::from_fn(|i| area[offset + i]))
    }
    fn read_bytes_at(&self, offset: usize, len: usize, what: &str) -> Result<Vec<u8>, BadInput> {
        self.read_at(offset, len, what)
    }
    fn write_bytes_at(&self, offset: usize, data: &[u8], what: &str) -> Result<(), BadInput> {
        self.write_at(offset, data, what)
    }
}

impl<const N: usize> From<&'static NameArea<N>> for BytePlace {
    fn from(area: &'static NameArea<N>) -> Self {
        BytePlace::In(area, 0)
    }
}

/// The names and the text backup.
pub struct Names {
    /// The four players' names, 0x20 bytes each (`menu::player_names`).
    pub player_names: NameArea<128>,
    /// The rest of the names' area.
    pub more_names: NameArea<128>,
    /// The name kept by `menu::standings` (0x1F bytes and a NUL).
    pub kept_name: NameArea<32>,
    /// The text as it was before the entry started (`menu::text_entry`).
    pub text_backup: NameArea<32>,
}

/// The whole area as one run of bytes: the four names, the rest of the names, the kept name and
/// the text backup (a name that lacks its end runs on into what follows it).
impl Run for Names {
    fn len(&self) -> usize {
        0x140
    }
    fn get(&self, n: usize) -> u8 {
        match n {
            0..0x80 => self.player_names.get(n),
            0x80..0x100 => self.more_names.get(n - 0x80),
            0x100..0x120 => self.kept_name.get(n - 0x100),
            _ => self.text_backup.get(n - 0x120),
        }
    }
    fn set(&self, n: usize, v: u8) {
        match n {
            0..0x80 => self.player_names.set(n, v),
            0x80..0x100 => self.more_names.set(n - 0x80, v),
            0x100..0x120 => self.kept_name.set(n - 0x100, v),
            _ => self.text_backup.set(n - 0x120, v),
        }
    }
}

impl Names {
    pub const fn new() -> Self {
        Names {
            player_names: NameArea::new(),
            more_names: NameArea::new(),
            kept_name: NameArea::new(),
            text_backup: NameArea::new(),
        }
    }
}
