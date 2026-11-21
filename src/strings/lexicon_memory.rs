//! The lexicon compiler's memory and the sizes of its work areas, which the strings' owner holds.

use crate::memory::MemoryList;
use crate::memory::session::PlainDword;

/// The work variables' size; they are written here and read by nothing.
pub const WORK_LEN: usize = 0xC44;
/// The lexicon compiler's memory (the glyph builder takes the same work memory).
pub struct LexiconMemory {
    /// The memory's kilobytes (the `imageheapsize` setting; the glyph tables' work memory is sized
    /// by it too).
    pub kilobytes: PlainDword,
    /// `LexiconMemory.memory.head`: the list the memory is taken from (16 bytes).
    pub memory: crate::memory::PlainList,
}

impl LexiconMemory {
    /// The lexicon's memory.
    pub const fn new() -> Self {
        LexiconMemory {
            kilobytes: PlainDword::new(0x400),
            memory: crate::memory::PlainList::new(MemoryList {
                head: 0,
                total: 0,
                cursor: 0,
                remaining: 0,
            }),
        }
    }
}

/// The record all empty messages share: how often it was used (never reset) and its index
/// entry. Written here only.
pub(super) const EMPTY_MESSAGE_LEN: usize = 0x10;
