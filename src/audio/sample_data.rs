//! The sounds' sample data in Rust's memory: the work buffer a sound file is read into, the
//! stream's header buffer, and the samples a voice keeps (`audio::keep_sample`). A voice's `data`
//! is an id into the table, its place + 1: a place in the work buffer, or a kept copy the entry owns.
//! Voices sharing a sound share the id. The entries go with the voices (`audio::free_voices`).
//! The memory is sized in whole pages less a header, so a read just past a sample's end reads
//! zeros and is not refused; the memory figures count it (`memory::count_owned`).

use crate::errors::BadInput;
use std::sync::Mutex;

/// What an id names.
enum Data {
    /// The work buffer from `offset` on.
    Work { offset: u32 },
    /// The stream's header buffer from `offset` on.
    Header { offset: u32 },
    /// A sample of its own.
    Kept(Vec<u8>),
}

/// The id table and the two buffers; one per audio thread (`AudioOwn::sample_data`).
pub struct SampleData {
    table: Mutex<Vec<Data>>,
    work: Mutex<Vec<u8>>,
    header: Mutex<Vec<u8>>,
}

/// The bytes a block of `bytes` gives: its whole pages less the header.
fn block_bytes(bytes: u32) -> usize {
    let header = rc2k_core::memory::HEADER_LEN as usize;
    (bytes as usize + header).next_multiple_of(0x1000) - header
}

/// Which buffer a sound file is read into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buffer {
    Work,
    Header,
}

/// Every id's bytes readable at once (the mixer's): the locks held while it lives, taken in
/// `with_data`'s order.
pub struct View<'a> {
    table: std::sync::MutexGuard<'a, Vec<Data>>,
    work: std::sync::MutexGuard<'a, Vec<u8>>,
    header: std::sync::MutexGuard<'a, Vec<u8>>,
}

impl View<'_> {
    /// Id `id`'s bytes, from its start to its memory's end.
    pub fn bytes(&self, id: u32) -> Result<&[u8], BadInput> {
        let (all, offset) = match self.table.get((id as usize).wrapping_sub(1)) {
            Some(Data::Kept(bytes)) => return Ok(bytes),
            Some(Data::Work { offset }) => (&self.work, *offset),
            Some(Data::Header { offset }) => (&self.header, *offset),
            None => return Err(BadInput("a sample data id that names nothing")),
        };
        all.get(offset as usize..)
            .ok_or(BadInput("a sample's data past its buffer"))
    }
}

fn locked<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl Default for SampleData {
    fn default() -> Self {
        Self::new()
    }
}

impl SampleData {
    pub const fn new() -> Self {
        SampleData {
            table: Mutex::new(Vec::new()),
            work: Mutex::new(Vec::new()),
            header: Mutex::new(Vec::new()),
        }
    }

    fn buffer(&self, which: Buffer) -> &Mutex<Vec<u8>> {
        match which {
            Buffer::Work => &self.work,
            Buffer::Header => &self.header,
        }
    }

    /// A buffer taken anew, zeroed, for `bytes` (`take_sample_buffer`'s, the header's).
    pub fn take(&self, which: Buffer, bytes: u32) {
        *locked(self.buffer(which)) = vec![0; block_bytes(bytes)];
    }

    /// A buffer given back.
    pub fn free(&self, which: Buffer) {
        *locked(self.buffer(which)) = Vec::new();
    }

    /// `f` on a buffer's bytes.
    pub fn with_buffer<R>(&self, which: Buffer, f: impl FnOnce(&mut [u8]) -> R) -> R {
        f(&mut locked(self.buffer(which)))
    }

    fn add(&self, data: Data) -> u32 {
        let mut table = locked(&self.table);
        table.push(data);
        table.len() as u32
    }

    /// An id for a buffer's bytes from `offset` on (a sound file's data where it was read).
    pub(super) fn in_buffer(&self, which: Buffer, offset: u32) -> u32 {
        self.add(match which {
            Buffer::Work => Data::Work { offset },
            Buffer::Header => Data::Header { offset },
        })
    }

    /// Where id `id`'s bytes start in its buffer (0 for a kept sample).
    pub fn offset(&self, id: u32) -> Result<u32, BadInput> {
        let table = locked(&self.table);
        match table.get((id as usize).wrapping_sub(1)) {
            Some(Data::Work { offset } | Data::Header { offset }) => Ok(*offset),
            Some(Data::Kept(_)) => Ok(0),
            None => Err(BadInput("a sample data id that names nothing")),
        }
    }

    /// `f` on id `id`'s bytes: from its start to its memory's end. Refuses an id that names
    /// nothing, or a buffer place past the buffer.
    pub(super) fn with_data<R>(
        &self,
        id: u32,
        f: impl FnOnce(&mut [u8]) -> R,
    ) -> Result<R, BadInput> {
        let mut table = locked(&self.table);
        let entry = table
            .get_mut((id as usize).wrapping_sub(1))
            .ok_or(BadInput("a sample data id that names nothing"))?;
        let (which, offset) = match entry {
            Data::Kept(bytes) => return Ok(f(bytes)),
            Data::Work { offset } => (Buffer::Work, *offset),
            Data::Header { offset } => (Buffer::Header, *offset),
        };
        let mut bytes = locked(self.buffer(which));
        let bytes = bytes
            .get_mut(offset as usize..)
            .ok_or(BadInput("a sample's data past its buffer"))?;
        Ok(f(bytes))
    }

    /// `f` with every id's bytes readable.
    pub(super) fn with_view<R>(&self, f: impl FnOnce(&View) -> R) -> R {
        let table = locked(&self.table);
        let work = locked(&self.work);
        let header = locked(&self.header);
        f(&View {
            table,
            work,
            header,
        })
    }

    /// `length` bytes of id `id` kept as a sample of its own (`audio::keep_sample`): its id.
    /// Counted on `list` as `memory::allocate` counts a block.
    pub fn keep(
        &self,
        id: u32,
        length: u32,
        list: &crate::memory::ListCells,
    ) -> Result<u32, BadInput> {
        let mut kept = vec![0; block_bytes(length)];
        self.with_data(id, |from| {
            let from = from
                .get(..length as usize)
                .ok_or(BadInput("a sample longer than its memory"))?;
            kept[..length as usize].copy_from_slice(from);
            Ok::<_, BadInput>(())
        })??;
        crate::memory::count_owned(list, length);
        Ok(self.add(Data::Kept(kept)))
    }

    /// Every id forgotten (the voices are gone: `audio::free_voices`).
    pub(super) fn forget_all(&self) {
        locked(&self.table).clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_is_found_by_id_kept_and_read_past_its_end() {
        let _lock = crate::test_support::test_lock();
        let data = SampleData::new();
        let figures = crate::memory::figures();
        data.take(Buffer::Work, 0x20);
        data.with_buffer(Buffer::Work, |b| b[4..8].copy_from_slice(&[1, 2, 3, 4]));
        let id = data.in_buffer(Buffer::Work, 4);
        assert_eq!(
            data.with_data(id, |b| b[..4].to_vec()).unwrap(),
            [1, 2, 3, 4]
        );
        let list = crate::memory::ListCells::empty();
        let kept = data.keep(id, 4, &list).unwrap();
        data.with_buffer(Buffer::Work, |b| b[4] = 9);
        assert_eq!(
            data.with_data(kept, |b| (b[..4].to_vec(), b.len()))
                .unwrap(),
            (vec![1, 2, 3, 4], 0x1000 - 0x40)
        );
        assert_eq!(list.get().total, 4);
        assert_eq!(
            data.with_data(id, |b| b[0]).unwrap(),
            9,
            "the work buffer is read as it is now"
        );
        assert!(data.with_data(0, |_| ()).is_err());
        crate::memory::with_figures(|f| *f = figures);
    }
}
