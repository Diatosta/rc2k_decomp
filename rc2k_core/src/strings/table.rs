//! The packed string table: what `lexicon::compile` makes, read back.
//!
//! An index of dwords, one per entry: the low 24 bits an offset into the table, the top byte
//! a flag. Below 0x80 the entry is that many bytes of text at the offset; from 0x80 it is
//! 0x100 less that many 16-bit entry numbers there, whose texts follow one another.

use crate::Invalid;

/// How deep entries may nest (the compiler makes two levels: messages of words).
const MAX_DEPTH: u32 = 8;

/// The table's bytes.
#[derive(Clone, Copy, Debug)]
pub struct StringTable<'a> {
    pub bytes: &'a [u8],
}

impl<'a> StringTable<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        StringTable { bytes }
    }

    fn entry(&self, id: u32) -> Option<u32> {
        let at = (id as usize).checked_mul(4)?;
        let b = self.bytes.get(at..at + 4)?;
        Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// The number of entries: the index ends where entry 0's data begins.
    pub fn entry_count(&self) -> usize {
        self.entry(0)
            .map_or(0, |e| ((e & 0x00FF_FFFF) / 4) as usize)
    }

    /// Entry `id`'s text appended to `out`; on a bad entry (past the index, data past the
    /// table, nesting too deep) `out` is left as it was.
    pub fn expand(&self, id: u32, out: &mut Vec<u8>) -> Result<(), Invalid> {
        let start = out.len();
        let done = self.expand_at(id, out, 0);
        if done.is_err() {
            out.truncate(start);
        }
        done
    }

    fn expand_at(&self, id: u32, out: &mut Vec<u8>, depth: u32) -> Result<(), Invalid> {
        if depth > MAX_DEPTH {
            return Err(Invalid("string table entries nested too deep"));
        }
        if id as usize >= self.entry_count() {
            return Err(Invalid("a string id past the string table's index"));
        }
        let entry = self
            .entry(id)
            .ok_or(Invalid("a string id past the table"))?;
        let flag = (entry >> 24) as usize;
        let data = (entry & 0x00FF_FFFF) as usize;
        let past = Invalid("a string table entry whose data is past the table");
        if flag < 0x80 {
            out.extend_from_slice(self.bytes.get(data..data + flag).ok_or(past)?);
        } else {
            let children = self
                .bytes
                .get(data..data + 2 * (0x100 - flag))
                .ok_or(past)?;
            for child in children.chunks(2) {
                self.expand_at(
                    u16::from_le_bytes([child[0], child[1]]) as u32,
                    out,
                    depth + 1,
                )?;
            }
        }
        Ok(())
    }

    pub fn text(&self, id: u32) -> Result<Vec<u8>, Invalid> {
        let mut out = Vec::new();
        self.expand(id, &mut out)?;
        Ok(out)
    }
}

/// String `index` of a list of NUL-terminated strings, without its NUL; an `index` of 0 or less
/// is the first.
pub fn nth_string(list: &[u8], index: i32) -> &[u8] {
    let mut rest = list;
    for _ in 0..index.max(0) {
        rest = match rest.iter().position(|&c| c == 0) {
            Some(nul) => &rest[nul + 1..],
            None => &[],
        };
    }
    let end = rest.iter().position(|&c| c == 0).unwrap_or(rest.len());
    &rest[..end]
}

/// The ids whose text may come from the second table of pointers (ids 0x5A5 ..= 0x658).
pub const POINTER_IDS: std::ops::RangeInclusive<u32> = 0x5A5..=0x658;

/// Where a string id's text is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// Ids with the 0x1000 bit: a table of pointers the game fills in (texts made at run
    /// time).
    Made,
    /// `POINTER_IDS` whose pointer is set: texts the game put there.
    Pointer,
    /// `own::FIRST ..= own::LAST`: the texts of the pages this game adds (`strings::own`).
    Own,
    /// Every other id: the string table.
    Table,
}

impl Source {
    /// `pointer_set` says whether the second pointer table has an entry for the id.
    pub fn of(id: u32, pointer_set: impl FnOnce() -> bool) -> Self {
        if crate::strings::own::is_own(id) {
            Source::Own
        } else if id & 0x1000 != 0 {
            Source::Made
        } else if POINTER_IDS.contains(&id) && pointer_set() {
            Source::Pointer
        } else {
            Source::Table
        }
    }
}

/// The text of `count` ids from `first`, with its NUL. `append` appends one id's text and returns
/// the id it finished at; the next id is that plus one (the id itself for the string table's
/// texts). `None` for a count of 0 or a text over `room`.
pub fn text_from_ids(
    first: u32,
    count: u32,
    room: usize,
    mut append: impl FnMut(u32, &mut Vec<u8>) -> u32,
) -> Option<Vec<u8>> {
    if count == 0 {
        return None;
    }
    let mut out = Vec::new();
    let mut id = first;
    for _ in 0..count {
        id = append(id, &mut out).wrapping_add(1);
    }
    out.push(0);
    (out.len() <= room).then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strings::lexicon::compile;

    const FILE: &[u8] = b"; a language file\r\n\
        HELLO\tHello, world\r\n\
        BYE\tGoodbye, world\r\n\
        ONE\tHello\r\n";

    #[test]
    fn compiled_messages_read_back() {
        let compiled = compile(FILE, 0x10_0000).unwrap();
        assert_eq!(compiled.messages, 3);
        let table = StringTable::new(&compiled.table);
        assert_eq!(table.text(0).unwrap(), b"Hello, world");
        assert_eq!(table.text(1).unwrap(), b"Goodbye, world");
        assert_eq!(table.text(2).unwrap(), b"Hello");
        assert!(table.entry_count() > 3);
    }

    #[test]
    fn helpers() {
        assert_eq!(nth_string(b"one\0two\0three\0", 0), b"one");
        assert_eq!(nth_string(b"one\0two\0three\0", 2), b"three");
        assert_eq!(nth_string(b"one\0two\0", -3), b"one");
        assert_eq!(nth_string(b"one\0", 5), b"");
        assert_eq!(Source::of(0x1001, || false), Source::Made);
        assert_eq!(Source::of(0x2001, || true), Source::Own);
        assert_eq!(Source::of(0x5A5, || true), Source::Pointer);
        assert_eq!(Source::of(0x5A5, || false), Source::Table);
        assert_eq!(Source::of(0x10, || true), Source::Table);
        let compiled = compile(FILE, 0x10_0000).unwrap();
        let table = StringTable::new(&compiled.table);
        let append = |id: u32, out: &mut Vec<u8>| {
            table.expand(id, out).unwrap();
            id
        };
        assert_eq!(
            text_from_ids(0, 2, 100, append).unwrap(),
            b"Hello, worldGoodbye, world\0"
        );
        assert_eq!(text_from_ids(0, 0, 100, append), None);
        assert_eq!(text_from_ids(0, 2, 10, append), None);
    }

    #[test]
    fn the_lexicon_corrections() {
        let mut file = Vec::new();
        for n in 0..0x6CB {
            let text: &[u8] = match n {
                0x6CA => b"Bowland Rally",
                0x6AB => b"Laukennen drives",
                0x23A => b"the A8",
                0x23B => b"the A7 road",
                _ => b"filler",
            };
            file.extend_from_slice(format!("M{n}\t").as_bytes());
            file.extend_from_slice(text);
            file.extend_from_slice(b"\r\n");
        }
        let mut compiled = crate::strings::lexicon::compile(&file, 0x100_0000).unwrap();
        crate::strings::lexicon::correct(&mut compiled.table).unwrap();
        let table = StringTable::new(&compiled.table);
        assert_eq!(table.text(0x6CA).unwrap(), b"Boland Rally");
        assert_eq!(table.text(0x6AB).unwrap(), b"Laukkanen drives");
        assert_eq!(table.text(0x23B).unwrap(), b"the A8 road");
        assert_eq!(table.text(0x23A).unwrap(), b"the A8");
    }

    #[test]
    fn bad_entries_are_refused() {
        let compiled = compile(FILE, 0x10_0000).unwrap();
        let table = StringTable::new(&compiled.table);
        let count = table.entry_count() as u32;
        let mut out = b"kept".to_vec();
        assert!(table.expand(count, &mut out).is_err());
        assert_eq!(out, b"kept");
        // An entry pointing at itself: refused, not a runaway.
        let mut looped = vec![0u8; 8];
        looped[0..4].copy_from_slice(&(0xFF00_0004u32).to_le_bytes());
        looped[4..6].copy_from_slice(&0u16.to_le_bytes());
        assert!(StringTable::new(&looped).text(0).is_err());
        assert!(StringTable::new(&[]).text(0).is_err());
    }
}
