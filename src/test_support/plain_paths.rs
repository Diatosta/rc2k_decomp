//! A plain record reached by field path, for the tests that name a field the way the held races'
//! hash does (`.cars[3].gear`), as `paths` does for records of cells. A path ending in an index
//! names one word of the array it follows; any other path names a whole run of like words (a
//! scalar's one word, an array in one piece). Words are read and written as little-endian
//! bytes, a float by its bits.

use crate::memory::cells::CellVisitor;
use crate::memory::plain_record::PlainRecord;

/// The runs of a record's walk: each run's path (an array as one run is at its element 0), where
/// its bytes start in the walk's flat bytes, and the flat bytes.
#[derive(Default)]
struct Walked {
    path: String,
    marks: Vec<usize>,
    runs: Vec<(String, usize, usize)>,
    flat: Vec<u8>,
}

impl CellVisitor for Walked {
    fn enter(&mut self, name: &'static str) {
        self.marks.push(self.path.len());
        self.path.push('.');
        self.path.push_str(name);
    }

    fn index(&mut self, i: usize) {
        self.marks.push(self.path.len());
        self.path.push_str(&format!("[{i}]"));
    }

    fn leave(&mut self) {
        let mark = self.marks.pop().expect("a part entered");
        self.path.truncate(mark);
    }

    fn leaf(&mut self, bytes: &[u8]) {
        self.runs
            .push((self.path.clone(), self.flat.len(), bytes.len()));
        self.flat.extend_from_slice(bytes);
    }
}

fn walked<T: PlainRecord>(record: &T) -> Walked {
    let mut walked = Walked::default();
    record.visit(&mut walked);
    walked
}

/// Where the bytes `path` names lie in the walk's flat bytes: the offset and the length.
fn find(runs: &[(String, usize, usize)], path: &str) -> Option<(usize, usize)> {
    let element = |path: &str| -> Option<(String, usize)> {
        let (base, index) = path.strip_suffix(']')?.rsplit_once('[')?;
        Some((base.to_string(), index.parse().ok()?))
    };
    // A path ending in an index names one word of the array that run holds.
    if let Some((base, index)) = element(path) {
        for (run, at, len) in runs {
            if (run == &format!("{base}[0]") || run == &base) && 4 * (index + 1) <= *len {
                return Some((at + 4 * index, 4));
            }
        }
    }
    runs.iter()
        .find(|(run, ..)| run == path || run.strip_suffix("[0]") == Some(path))
        .map(|&(_, at, len)| (at, len))
}

/// Every byte of the record, in the order of its walk.
pub fn flat_bytes<T: PlainRecord>(record: &T) -> Vec<u8> {
    walked(record).flat
}

/// The record set from flat bytes in the order of its walk.
pub fn set_flat_bytes<T: PlainRecord>(record: &mut T, bytes: &[u8]) {
    let mut at = 0;
    record.fill(bytes, &mut at);
}

/// The offset in the flat bytes of the words `path` names.
pub fn flat_offset<T: PlainRecord>(record: &T, path: &str) -> Option<usize> {
    Some(find(&walked(record).runs, path)?.0)
}

/// The bytes `path` names, or none when no field has that path.
pub fn bytes<T: PlainRecord>(record: &T, path: &str) -> Option<Vec<u8>> {
    let walked = walked(record);
    let (at, len) = find(&walked.runs, path)?;
    Some(walked.flat[at..at + len].to_vec())
}

/// The bytes `path` names set from `bytes`; says whether the path and the length fitted.
pub fn set_bytes<T: PlainRecord>(record: &mut T, path: &str, bytes: &[u8]) -> bool {
    let mut walked = walked(&*record);
    let Some((at, len)) = find(&walked.runs, path) else {
        return false;
    };
    if len != bytes.len() {
        return false;
    }
    walked.flat[at..at + len].copy_from_slice(bytes);
    set_flat_bytes(record, &walked.flat);
    true
}

/// The four-byte word at `path` (a float by its bits).
pub fn get_u32<T: PlainRecord>(record: &T, path: &str) -> Option<u32> {
    Some(u32::from_le_bytes(bytes(record, path)?.try_into().ok()?))
}

/// The four-byte word at `path` set; says whether there was one.
pub fn set_u32<T: PlainRecord>(record: &mut T, path: &str, value: u32) -> bool {
    set_bytes(record, path, &value.to_le_bytes())
}
