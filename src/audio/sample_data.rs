//! The sounds' sample data in Rust's memory (phase 12; it was the game's heap, reached by
//! address): the work buffer a sound file is read into (`g_sampleBuffer`, its size
//! `g_sampleBufferSize`), the stream's header buffer (`g_streamHeader`), and the samples a
//! voice keeps (`audio::keep_sample`, on `g_sampleMemory`'s list).
//!
//! A voice's `data` (a channel's, the sample file's) is an id into `DATA`, its place + 1:
//! a place in the work buffer, or a kept copy the entry owns. Voices sharing a sound share
//! the id. A voice that does not keep its sample reads the work buffer where its sound was
//! read, whatever was read there since, as the original does. The entries go with the
//! voices (`audio::free_voices`).
//!
//! The memory is sized as the game's blocks were (whole pages, the block's header left
//! out): a read just past a sample's end reads zeros, as in the game, and is not
//! refused. The memory figures count it as `memory::allocate` did (`memory::count_owned`).

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

static DATA: Mutex<Vec<Data>> = Mutex::new(Vec::new());
static WORK: Mutex<Vec<u8>> = Mutex::new(Vec::new());
static HEADER: Mutex<Vec<u8>> = Mutex::new(Vec::new());

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

fn buffer(which: Buffer) -> &'static Mutex<Vec<u8>> {
    match which {
        Buffer::Work => &WORK,
        Buffer::Header => &HEADER,
    }
}

/// A buffer taken anew, zeroed, for `bytes` (`take_sample_buffer`'s, the header's).
pub fn take(which: Buffer, bytes: u32) {
    *buffer(which)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = vec![0; block_bytes(bytes)];
}

/// A buffer given back.
pub fn free(which: Buffer) {
    *buffer(which)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Vec::new();
}

/// `f` on a buffer's bytes.
pub fn with_buffer<R>(which: Buffer, f: impl FnOnce(&mut [u8]) -> R) -> R {
    f(&mut buffer(which)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner))
}

fn add(data: Data) -> u32 {
    let mut table = DATA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    table.push(data);
    table.len() as u32
}

/// An id for a buffer's bytes from `offset` on (a sound file's data where it was read).
pub fn in_buffer(which: Buffer, offset: u32) -> u32 {
    add(match which {
        Buffer::Work => Data::Work { offset },
        Buffer::Header => Data::Header { offset },
    })
}

/// Where id `id`'s bytes start in its buffer (0 for a kept sample).
pub fn offset(id: u32) -> Result<u32, BadInput> {
    let table = DATA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match table.get((id as usize).wrapping_sub(1)) {
        Some(Data::Work { offset } | Data::Header { offset }) => Ok(*offset),
        Some(Data::Kept(_)) => Ok(0),
        None => Err(BadInput("a sample data id that names nothing")),
    }
}

/// `f` on id `id`'s bytes: from its start to its memory's end. Refuses an id that names
/// nothing, or a buffer place past the buffer.
pub fn with_data<R>(id: u32, f: impl FnOnce(&mut [u8]) -> R) -> Result<R, BadInput> {
    let mut table = DATA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let entry = table
        .get_mut((id as usize).wrapping_sub(1))
        .ok_or(BadInput("a sample data id that names nothing"))?;
    let (which, offset) = match entry {
        Data::Kept(bytes) => return Ok(f(bytes)),
        Data::Work { offset } => (Buffer::Work, *offset),
        Data::Header { offset } => (Buffer::Header, *offset),
    };
    let mut bytes = buffer(which)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let bytes = bytes
        .get_mut(offset as usize..)
        .ok_or(BadInput("a sample's data past its buffer"))?;
    Ok(f(bytes))
}

/// Every id's bytes readable at once (the mixer's): the locks held while it lives, taken
/// in `with_data`'s order.
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

/// `f` with every id's bytes readable.
pub fn with_view<R>(f: impl FnOnce(&View) -> R) -> R {
    let table = DATA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let work = WORK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let header = HEADER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    f(&View {
        table,
        work,
        header,
    })
}

/// `length` bytes of id `id` kept as a sample of its own (`audio::keep_sample`): its id.
/// Counted on `list` as `memory::allocate` counts a block.
pub fn keep(
    id: u32,
    length: u32,
    list: &crate::session::Shared<crate::memory::MemoryList>,
) -> Result<u32, BadInput> {
    let mut kept = vec![0; block_bytes(length)];
    with_data(id, |from| {
        let from = from
            .get(..length as usize)
            .ok_or(BadInput("a sample longer than its memory"))?;
        kept[..length as usize].copy_from_slice(from);
        Ok::<_, BadInput>(())
    })??;
    crate::memory::count_owned(list, length);
    Ok(add(Data::Kept(kept)))
}

/// Every id forgotten (the voices are gone: `audio::free_voices`).
pub fn forget_all() {
    DATA.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_is_found_by_id_kept_and_read_past_its_end() {
        let _lock = crate::store::test_lock();
        let was = std::mem::take(&mut *WORK.lock().unwrap());
        let figures = crate::memory::figures();
        take(Buffer::Work, 0x20);
        with_buffer(Buffer::Work, |b| b[4..8].copy_from_slice(&[1, 2, 3, 4]));
        let id = in_buffer(Buffer::Work, 4);
        assert_eq!(with_data(id, |b| b[..4].to_vec()).unwrap(), [1, 2, 3, 4]);
        let list = crate::session::Shared::new(crate::memory::MemoryList::default());
        let kept = keep(id, 4, &list).unwrap();
        with_buffer(Buffer::Work, |b| b[4] = 9);
        assert_eq!(
            with_data(kept, |b| (b[..4].to_vec(), b.len())).unwrap(),
            (vec![1, 2, 3, 4], 0x1000 - 0x40)
        );
        assert_eq!(list.get().total, 4);
        assert_eq!(
            with_data(id, |b| b[0]).unwrap(),
            9,
            "the work buffer is read as it is now"
        );
        assert!(with_data(0, |_| ()).is_err());
        *WORK.lock().unwrap() = was;
        crate::memory::with_figures(|f| *f = figures);
    }
}
