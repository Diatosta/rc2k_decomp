//! The keyboard layouts of a `.kim` file, as owned tables.
//!
//! The file's 0x400 bytes: "KEYB", +8 a kind, +0xA the layout mode, +0xB the picked layout,
//! +0x30 the layouts, five dwords each: offsets, from +0x30, of the dead keys' list, the plain,
//! shifted and AltGr tables and the key names' table (a word, a string id, a key). A table runs
//! to the end of the block; a read past it is refused. A kind 0x0C file has three entries of
//! the picked layout's key names table patched.
//!
//! Offsets are signed in effect (the addition wraps): one landing inside the block, head
//! included, is a table; one outside it is `Table::Outside`, refused when the layout is used.

use super::keyboard::Layout as Tables;
use crate::Invalid;

/// The file's length.
pub const FILE_LEN: usize = 0x400;
const LAYOUTS_AT: usize = 0x30;
const LAYOUT_LEN: usize = 0x14;
/// The kind whose picked layout gets three key names patched.
const PATCHED_KIND: u8 = 0x0C;
/// (offset in the key names' table, string id) of the patched entries.
const PATCHES: [(usize, u16); 3] = [(0x34, 0x1034), (0x36, 0x1035), (0x52, 0x1036)];

/// One of a layout's tables.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Table {
    /// No table (a null pointer).
    #[default]
    Absent,
    /// A pointer that names memory outside the file.
    Outside,
    Bytes(Vec<u8>),
}

impl Table {
    fn bytes(&self) -> Result<Option<&Vec<u8>>, Invalid> {
        match self {
            Table::Absent => Ok(None),
            Table::Outside => Err(Invalid("a keyboard layout's table outside the file")),
            Table::Bytes(bytes) => Ok(Some(bytes)),
        }
    }
}

/// A keyboard layout.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Layout {
    /// A list of {dead key, plain, shifted} ended by a 0 key.
    pub dead_keys: Table,
    pub plain: Table,
    pub shifted: Table,
    /// The table used while AltGr is down.
    pub alt_gr: Table,
    /// A string id (a word) a key; 0 for none.
    pub key_names: Table,
}

impl Layout {
    pub const EMPTY: Layout = Layout {
        dead_keys: Table::Absent,
        plain: Table::Absent,
        shifted: Table::Absent,
        alt_gr: Table::Absent,
        key_names: Table::Absent,
    };

    /// A layout of just a plain table, a shifted one and key names (the built-in ones).
    pub fn simple(plain: &[u8], shifted: &[u8], key_names: &[u8]) -> Layout {
        Layout {
            plain: Table::Bytes(plain.to_vec()),
            shifted: Table::Bytes(shifted.to_vec()),
            key_names: Table::Bytes(key_names.to_vec()),
            ..Layout::EMPTY
        }
    }

    /// The four tables `keyboard::key_to_char` reads. Refused if one names memory outside
    /// the file.
    pub fn tables(&self) -> Result<Tables<'_>, Invalid> {
        use crate::strings::bytes::Bytes;
        fn view(table: &Table) -> Result<Option<&dyn Bytes>, Invalid> {
            Ok(table.bytes()?.map(|b| b as &dyn Bytes))
        }
        Ok(Tables {
            dead_keys: view(&self.dead_keys)?,
            plain: view(&self.plain)?,
            shifted: view(&self.shifted)?,
            alt_gr: view(&self.alt_gr)?,
        })
    }

    /// The string id the key's low seven bits name (0: none).
    pub fn key_name_id(&self, key: u32) -> Result<u16, Invalid> {
        let names = match &self.key_names {
            Table::Absent => return Err(Invalid("a keyboard layout without its key names")),
            Table::Outside => return Err(Invalid("a keyboard layout's table outside the file")),
            Table::Bytes(names) => names,
        };
        let at = (key & 0x7F) as usize * 2;
        match names.get(at..at + 2) {
            Some(word) => Ok(u16::from_le_bytes([word[0], word[1]])),
            None => Err(Invalid("a key's name id past its table")),
        }
    }
}

/// Which layout of the file is the current one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Picked {
    /// The first or the second of the file's layouts (0 or 1).
    Listed(usize),
    /// The byte named a layout past the two the file lists: none of its tables can be read.
    Unlisted(Layout),
}

/// What the file gives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct File {
    /// The layout mode (2: Alt with a shift key switches the layout).
    pub mode: u8,
    pub layouts: [Layout; 2],
    pub picked: Picked,
    /// The 0x400 bytes after the kind 0x0C patch (kept by the caller: a later, shorter file
    /// is read over them).
    pub bytes: [u8; FILE_LEN],
}

/// Where a table starts in the file, if the offset names a place inside it.
fn place(offset: u32) -> Option<usize> {
    let at = LAYOUTS_AT as i64 + offset as i32 as i64;
    (0..FILE_LEN as i64).contains(&at).then_some(at as usize)
}

/// Where a layout's five tables are: `None` a null pointer, `Err` outside the file.
type Places = [Option<Result<usize, ()>>; 5];

/// The layout block `n` as places; `resolved` is false for a layout whose offsets are left
/// unresolved (one past the first two). `None` if the block does not fit in the file.
fn places(file: &[u8; FILE_LEN], n: usize, resolved: bool) -> Option<Places> {
    let start = LAYOUTS_AT + n * LAYOUT_LEN;
    let words = file.get(start..start + LAYOUT_LEN)?;
    Some(std::array::from_fn(|k| {
        let offset = u32::from_le_bytes(words[4 * k..4 * k + 4].try_into().unwrap());
        (offset != 0).then(|| match resolved {
            true => place(offset).ok_or(()),
            false => Err(()),
        })
    }))
}

fn table(file: &[u8; FILE_LEN], at: Option<Result<usize, ()>>) -> Table {
    match at {
        None => Table::Absent,
        Some(Err(())) => Table::Outside,
        Some(Ok(at)) => Table::Bytes(file[at..].to_vec()),
    }
}

fn layout(file: &[u8; FILE_LEN], places: Option<Places>) -> Layout {
    let Some(p) = places else {
        // A block that does not fit in the file: every pointer of it reads outside.
        return Layout {
            dead_keys: Table::Outside,
            plain: Table::Outside,
            shifted: Table::Outside,
            alt_gr: Table::Outside,
            key_names: Table::Outside,
        };
    };
    Layout {
        dead_keys: table(file, p[0]),
        plain: table(file, p[1]),
        shifted: table(file, p[2]),
        alt_gr: table(file, p[3]),
        key_names: table(file, p[4]),
    }
}

/// Reads a keyboard file's 0x400 bytes (its "KEYB" checked by the caller). Refused: a kind
/// 0x0C file whose picked layout has no key names table in the file to patch.
pub fn parse(file: &[u8; FILE_LEN]) -> Result<File, Invalid> {
    let kind = file[8];
    let mode = file[0xA];
    let pick = file[0xB] as usize;
    let listed = pick < 2;
    let mut bytes = *file;
    if kind == PATCHED_KIND {
        let names = places(file, pick, listed)
            .and_then(|p| p[4])
            .and_then(Result::ok)
            .ok_or(Invalid("a keyboard of kind 0x0C without its fourth table"))?;
        for (at, id) in PATCHES {
            let word = bytes
                .get_mut(names + at..names + at + 2)
                .ok_or(Invalid("a keyboard table"))?;
            word.copy_from_slice(&id.to_le_bytes());
        }
    }
    let listed_layout = |n| layout(&bytes, places(&bytes, n, true));
    Ok(File {
        mode,
        layouts: [listed_layout(0), listed_layout(1)],
        picked: if listed {
            Picked::Listed(pick)
        } else {
            Picked::Unlisted(layout(&bytes, places(&bytes, pick, false)))
        },
        bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(f: &mut [u8; FILE_LEN], at: usize, v: u32) {
        f[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }

    /// A file with two layouts; the tables follow at 0x58.
    fn file(kind: u8, mode: u8, pick: u8) -> [u8; FILE_LEN] {
        let mut f = [0u8; FILE_LEN];
        f[..4].copy_from_slice(b"KEYB");
        f[8] = kind;
        f[0xA] = mode;
        f[0xB] = pick;
        // layout 0 (at 0x30): plain at 0x58, shifted at 0xD8, key names at 0x158
        word(&mut f, 0x34, 0x28);
        word(&mut f, 0x38, 0xA8);
        word(&mut f, 0x40, 0x128);
        // layout 1 (at 0x44): the same plain table only
        word(&mut f, 0x48, 0x28);
        for k in 0..0x80 {
            f[0x58 + k] = k as u8 + 1;
            f[0xD8 + k] = k as u8 + 2;
        }
        f[0x158 + 4] = 0x77; // key 2's name id
        f
    }

    #[test]
    fn tables_are_the_files_tails() {
        let f = parse(&file(1, 2, 1)).unwrap();
        assert_eq!(f.mode, 2);
        assert_eq!(f.picked, Picked::Listed(1));
        let first = f.layouts[0].tables().unwrap();
        assert_eq!(first.plain.unwrap().byte(0), Some(1));
        assert_eq!(first.shifted.unwrap().byte(0), Some(2));
        assert!(first.dead_keys.is_none() && first.alt_gr.is_none());
        assert_eq!(f.layouts[0].key_name_id(2), Ok(0x77));
        assert_eq!(f.layouts[0].key_name_id(0x82), Ok(0x77));
        assert_eq!(f.layouts[0].key_name_id(1), Ok(0));
        assert_eq!(
            f.layouts[1].key_name_id(2),
            Err(Invalid("a keyboard layout without its key names"))
        );
        // The tail ends with the file.
        match &f.layouts[0].plain {
            Table::Bytes(b) => assert_eq!(b.len(), FILE_LEN - 0x58),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn offsets_outside_the_file_are_outside() {
        let mut bytes = file(1, 1, 0);
        word(&mut bytes, 0x34, 0x400);
        word(&mut bytes, 0x38, -0x30i32 as u32);
        let f = parse(&bytes).unwrap();
        assert_eq!(f.layouts[0].plain, Table::Outside);
        assert!(f.layouts[0].tables().is_err());
        // -0x30 is the file's first byte.
        match &f.layouts[0].shifted {
            Table::Bytes(b) => assert_eq!(&b[..4], b"KEYB"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn kind_0c_patches_three_names() {
        let mut bytes = file(0x0C, 1, 0);
        word(&mut bytes, 0x54, 0x128); // layout 1 shares the key names
        let f = parse(&bytes).unwrap();
        for l in &f.layouts {
            assert_eq!(l.key_name_id(0x34 / 2), Ok(0x1034));
            assert_eq!(l.key_name_id(0x36 / 2), Ok(0x1035));
            assert_eq!(l.key_name_id(0x52 / 2), Ok(0x1036));
        }
        // Without the table, or with the pick outside the two, it is refused.
        assert!(parse(&file(0x0C, 1, 1)).is_err());
        assert!(parse(&file(0x0C, 1, 2)).is_err());
    }

    #[test]
    fn a_pick_past_the_two_is_unlisted() {
        let mut bytes = file(1, 1, 3);
        // Layout 3 is at 0x6C (over the fixture's plain table): one raw offset, the rest null.
        bytes[0x6C..0x80].fill(0);
        word(&mut bytes, 0x6C, 5);
        let f = parse(&bytes).unwrap();
        match f.picked {
            Picked::Unlisted(l) => {
                assert_eq!(l.dead_keys, Table::Outside);
                assert_eq!(l.plain, Table::Absent);
            }
            other => panic!("{other:?}"),
        }
        // A block not inside the file reads outside throughout.
        let f = parse(&file(1, 1, 0xFF)).unwrap();
        match f.picked {
            Picked::Unlisted(l) => assert_eq!(l.key_names, Table::Outside),
            other => panic!("{other:?}"),
        }
    }
}
