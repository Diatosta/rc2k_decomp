//! A record's cells reached by field path, for the tests that name a field the way the debug
//! pages and the held races' hash do (`.cars[3].gear`; a field called `inline` adds nothing)
//! instead of by byte offset or address. Any `Cells` record will do, whatever its layout.
//!
//! A path ending in an index (`.gears[2]`) names one cell of the array it follows; any other
//! path names a whole run of like cells (a scalar's one cell, a byte array in one piece). A
//! cell is read and written as little-endian bytes, a float by its bits.

use crate::memory::cells::{CellRun, CellVisitor, Cells};

/// What the walk does at the run it finds.
enum Task<'a> {
    /// The cells' bytes, copied out.
    Get(&'a mut Option<Vec<u8>>),
    /// The cells' bytes, replaced by these.
    Set(&'a [u8], &'a mut bool),
    /// The address of the first cell.
    Addr(&'a mut Option<usize>),
}

/// The path split into the part before a trailing index and that index.
fn split_index(path: &str) -> Option<(&str, usize)> {
    let inner = path.strip_suffix(']')?;
    let (base, index) = inner.rsplit_once('[')?;
    Some((base, index.parse().ok()?))
}

struct Finder<'a> {
    want: &'a str,
    /// The paths a run named by `want` can be at (`cells`): `want`, or `want[0]` (an array
    /// reported as its element 0); for a `want` ending in an index, the array's own path and
    /// its `[0]`.
    targets: Vec<String>,
    path: String,
    /// The path's length before each part entered.
    marks: Vec<usize>,
    /// How deep the walk is below the last part that could still lead to a target: no run under it
    /// is one (the walk skips building their paths, as the race-block vectors walk whole stores per
    /// place and case).
    away: usize,
    task: Task<'a>,
    done: bool,
}

impl<'a> Finder<'a> {
    fn new(want: &'a str, task: Task<'a>) -> Self {
        let mut targets = vec![want.to_string(), format!("{want}[0]")];
        if let Some((base, _)) = split_index(want) {
            targets.extend([base.to_string(), format!("{base}[0]")]);
        }
        Finder {
            want,
            targets,
            path: String::new(),
            marks: Vec::new(),
            away: 0,
            task,
            done: false,
        }
    }

    /// Whether some target starts with the current path: a run at it or under it can be one.
    fn leads_to_a_target(&self) -> bool {
        self.targets
            .iter()
            .any(|t| t.starts_with(self.path.as_str()))
    }

    /// The part `push` writes entered: kept on the path when it can lead to a target,
    /// otherwise counted in `away`.
    fn enter_part(&mut self, push: impl FnOnce(&mut String)) {
        if self.away > 0 {
            self.away += 1;
            return;
        }
        let mark = self.path.len();
        push(&mut self.path);
        if self.leads_to_a_target() {
            self.marks.push(mark);
        } else {
            self.path.truncate(mark);
            self.away = 1;
        }
    }
}

impl Finder<'_> {
    /// Which cells of the run at the current path the wanted path names: from the first cell
    /// on, `count` of them, or none.
    fn cells(&self, run: CellRun<'_>) -> Option<(usize, usize)> {
        let all = run.len() / run.width();
        if self.path == self.want {
            return match split_index(self.want) {
                Some(_) => Some((0, 1)),
                None => Some((0, all)),
            };
        }
        // An array of one kind of cell is one run, reported as its element 0: named without
        // an index it is the whole run.
        if split_index(self.want).is_none() && self.path.strip_suffix("[0]") == Some(self.want) {
            return Some((0, all));
        }
        let (base, index) = split_index(self.want)?;
        let whole = self.path == base || self.path.strip_suffix("[0]") == Some(base);
        (whole && index < all).then_some((index, 1))
    }
}

impl CellVisitor for Finder<'_> {
    fn enter(&mut self, name: &'static str) {
        self.enter_part(|path| {
            path.push('.');
            path.push_str(name);
        });
    }

    fn index(&mut self, i: usize) {
        use std::fmt::Write;
        self.enter_part(|path| {
            write!(path, "[{i}]").expect("a write into a String");
        });
    }

    fn leave(&mut self) {
        if self.away > 0 {
            self.away -= 1;
            return;
        }
        let mark = self.marks.pop().expect("a part entered");
        self.path.truncate(mark);
    }

    fn leaf(&mut self, _: &[u8]) {}

    fn run(&mut self, run: CellRun<'_>) {
        // A run away from every target is not one (`cells` would say none).
        if self.done || self.away > 0 || run.len() == 0 {
            return;
        }
        let Some((first, count)) = self.cells(run) else {
            return;
        };
        let (at, len) = (first * run.width(), count * run.width());
        self.done = true;
        match &mut self.task {
            Task::Get(out) => {
                let mut bytes = vec![0; len];
                run.load(at, &mut bytes);
                **out = Some(bytes);
            }
            Task::Set(bytes, done) => {
                if bytes.len() == len {
                    run.store(at, bytes);
                    **done = true;
                }
            }
            Task::Addr(out) => **out = Some(run.first() + at),
        }
    }
}

fn walk<T: Cells>(record: &T, path: &str, task: Task<'_>) {
    let mut finder = Finder::new(path, task);
    record.visit(&mut finder);
}

/// The bytes of the cells `path` names, or none when no cell has that path.
pub fn bytes<T: Cells>(record: &T, path: &str) -> Option<Vec<u8>> {
    let mut out = None;
    walk(record, path, Task::Get(&mut out));
    out
}

/// The cells `path` names set from `bytes` (as many as the cells hold); says whether the path
/// and the length fitted.
pub fn set_bytes<T: Cells>(record: &T, path: &str, bytes: &[u8]) -> bool {
    let mut done = false;
    walk(record, path, Task::Set(bytes, &mut done));
    done
}

/// The address of the first cell `path` names, for the tests that compare where two records
/// keep a field.
pub fn first_addr<T: Cells>(record: &T, path: &str) -> Option<usize> {
    let mut out = None;
    walk(record, path, Task::Addr(&mut out));
    out
}

/// The one-byte cell at `path`.
pub fn get_u8<T: Cells>(record: &T, path: &str) -> Option<u8> {
    Some(u8::from_le_bytes(bytes(record, path)?.try_into().ok()?))
}

/// The two-byte cell at `path`.
pub fn get_u16<T: Cells>(record: &T, path: &str) -> Option<u16> {
    Some(u16::from_le_bytes(bytes(record, path)?.try_into().ok()?))
}

/// The four-byte cell at `path` (a float by its bits).
pub fn get_u32<T: Cells>(record: &T, path: &str) -> Option<u32> {
    Some(u32::from_le_bytes(bytes(record, path)?.try_into().ok()?))
}

/// The float cell at `path`.
pub fn get_f32<T: Cells>(record: &T, path: &str) -> Option<f32> {
    get_u32(record, path).map(f32::from_bits)
}

/// The one-byte cell at `path` set; says whether there was one.
pub fn set_u8<T: Cells>(record: &T, path: &str, value: u8) -> bool {
    set_bytes(record, path, &value.to_le_bytes())
}

/// The two-byte cell at `path` set; says whether there was one.
pub fn set_u16<T: Cells>(record: &T, path: &str, value: u16) -> bool {
    set_bytes(record, path, &value.to_le_bytes())
}

/// The four-byte cell at `path` set; says whether there was one.
pub fn set_u32<T: Cells>(record: &T, path: &str, value: u32) -> bool {
    set_bytes(record, path, &value.to_le_bytes())
}

/// The float cell at `path` set; says whether there was one.
pub fn set_f32<T: Cells>(record: &T, path: &str, value: f32) -> bool {
    set_u32(record, path, value.to_bits())
}

/// Whether `head` followed by `tail` starts with `path`.
fn starts(head: &str, tail: &str, path: &str) -> bool {
    if path.len() <= head.len() {
        head.starts_with(path)
    } else {
        path.strip_prefix(head)
            .is_some_and(|rest| tail.starts_with(rest))
    }
}

/// What the walk of `field_bytes` and its relatives does at each run it takes in.
enum Span<'a> {
    /// The cells' bytes, appended in field order.
    Get(&'a mut Vec<u8>),
    /// The cells' bytes, replaced from these (the front is taken as it goes).
    Set(&'a mut &'a [u8]),
    /// The bytes of every cell before the first run taken in, and that run's cell width.
    Before(&'a mut Option<(usize, usize)>),
}

struct Under<'a> {
    prefix: &'a str,
    /// How many bytes of cells to take from the field's first cell on, past the field if need
    /// be; none: the cells of the field only.
    length: Option<usize>,
    path: String,
    marks: Vec<usize>,
    /// The bytes of the cells walked so far.
    seen: usize,
    /// The bytes taken in so far.
    taken: usize,
    started: bool,
    /// The field is a cell of an array of like cells, which the walk reports as one run (no
    /// run lies at the field's own path).
    in_run: bool,
    /// How deep the walk is below the last part that can matter (see `enter_part`).
    away: usize,
    task: Span<'a>,
}

impl Under<'_> {
    /// The part `push` writes entered: on the path when it can lead to a run `inside` the
    /// field or to the run `cell_of_run` looks for, otherwise counted in `away`.
    fn enter_part(&mut self, push: impl FnOnce(&mut String)) {
        if self.prefix.is_empty() {
            return;
        }
        if self.away > 0 {
            self.away += 1;
            return;
        }
        let mark = self.path.len();
        push(&mut self.path);
        let path = self.path.as_str();
        let leads = self.prefix.starts_with(path)
            || path.starts_with(self.prefix)
            || split_index(self.prefix).is_some_and(|(base, _)| starts(base, "[0]", path));
        if leads {
            self.marks.push(mark);
        } else {
            self.path.truncate(mark);
            self.away = 1;
        }
    }

    /// Whether the run at the current path is the field or lies in it (the empty path is the
    /// whole record).
    fn inside(&self) -> bool {
        if self.away > 0 {
            return false;
        }
        let Some(rest) = self.path.strip_prefix(self.prefix) else {
            return false;
        };
        self.prefix.is_empty() || rest.is_empty() || rest.starts_with(['.', '['])
    }

    /// Whether the run at the current path is an array of like cells whose cell `prefix` names
    /// (`.damage[2]`; the walk reports the array as its element 0), and the byte of the run the
    /// field starts at.
    fn cell_of_run(&self, run: &CellRun<'_>) -> Option<usize> {
        if !self.in_run || self.away > 0 {
            return None;
        }
        let (base, index) = split_index(self.prefix)?;
        (index > 0 && self.path.strip_suffix("[0]") == Some(base))
            .then(|| (index * run.width()).min(run.len()))
    }
}

// The walk builds only the paths that can matter (the vectors copy records of up to 2 MB this way
// in every case). The whole record (an empty `prefix`) needs none: every run is inside it
// (`inside`) and none is a cell of a run it names (`cell_of_run`). Otherwise a part whose path is
// neither a start of `prefix` (or of the run `cell_of_run` looks for) nor under `prefix` has no run
// under it that is either: it is counted in `away` and not written out.
impl CellVisitor for Under<'_> {
    fn enter(&mut self, name: &'static str) {
        self.enter_part(|path| {
            path.push('.');
            path.push_str(name);
        });
    }

    fn index(&mut self, i: usize) {
        use std::fmt::Write;
        self.enter_part(|path| {
            write!(path, "[{i}]").expect("a write into a String");
        });
    }

    fn leave(&mut self) {
        if self.prefix.is_empty() {
            return;
        }
        if self.away > 0 {
            self.away -= 1;
            return;
        }
        let mark = self.marks.pop().expect("a part entered");
        self.path.truncate(mark);
    }

    fn leaf(&mut self, _: &[u8]) {}

    fn run(&mut self, run: CellRun<'_>) {
        let len = run.len();
        if len == 0 {
            return;
        }
        let from = match self.length {
            Some(_) if !self.started => self.cell_of_run(&run),
            _ => None,
        };
        let skip = from.unwrap_or(0);
        let take = match self.length {
            None if self.inside() => len,
            Some(n) if self.started || self.inside() || from.is_some() => {
                (len - skip).min(n - self.taken)
            }
            _ => 0,
        };
        if take > 0 {
            self.started = true;
            match &mut self.task {
                Span::Get(out) => {
                    let at = out.len();
                    out.resize(at + take, 0);
                    run.load(skip, &mut out[at..]);
                }
                Span::Set(rest) => {
                    run.store(skip, &rest[..take]);
                    **rest = &rest[take..];
                }
                Span::Before(out) => {
                    out.get_or_insert((self.seen, run.width()));
                }
            }
            self.taken += take;
        }
        self.seen += len;
    }
}

impl<'a> Under<'a> {
    fn new(prefix: &'a str, length: Option<usize>, in_run: bool, task: Span<'a>) -> Self {
        Under {
            prefix,
            length,
            path: String::new(),
            marks: Vec::new(),
            seen: 0,
            taken: 0,
            started: false,
            in_run,
            away: 0,
            task,
        }
    }
}

/// Whether some run of the record lies in the field or record `prefix` names.
fn has_run_in<T: Cells>(record: &T, prefix: &str) -> bool {
    let mut found = None;
    record.visit(&mut Under::new(
        prefix,
        None,
        false,
        Span::Before(&mut found),
    ));
    found.is_some()
}

fn walk_under<'a, T: Cells>(record: &T, prefix: &'a str, length: Option<usize>, task: Span<'a>) {
    // A path ending in an index names a record of an array or, where the array is of like
    // cells, one cell of its single run: the second when no run lies at the path itself.
    let in_run = length.is_some()
        && split_index(prefix).is_some_and(|(_, index)| index > 0)
        && !has_run_in(record, prefix);
    record.visit(&mut Under::new(prefix, length, in_run, task));
}

/// The bytes of every cell in the field or record `prefix` names (the empty path is the whole
/// record), in field order: a field of several runs or a sub-record in one piece.
pub fn field_bytes<T: Cells>(record: &T, prefix: &str) -> Vec<u8> {
    let mut out = Vec::new();
    walk_under(record, prefix, None, Span::Get(&mut out));
    out
}

/// The `length` bytes of cells that begin with the field `prefix` names, in field order: the
/// field and as many of the fields after it as make up the length. Fewer bytes come back when
/// the record ends first.
pub fn field_range<T: Cells>(record: &T, prefix: &str, length: usize) -> Vec<u8> {
    let mut out = Vec::new();
    walk_under(record, prefix, Some(length), Span::Get(&mut out));
    out
}

/// The cells `field_range` names set from `bytes`; says whether the record held that many.
pub fn set_field_range<T: Cells>(record: &T, prefix: &str, bytes: &[u8]) -> bool {
    let fits = field_range(record, prefix, bytes.len()).len() == bytes.len();
    if fits {
        let mut rest = bytes;
        walk_under(record, prefix, Some(bytes.len()), Span::Set(&mut rest));
    }
    fits
}

/// How many bytes of cells come before the field `prefix` names in field order (a path ending
/// in an index names that cell of its array), or none when it has no cells.
pub fn field_start<T: Cells>(record: &T, prefix: &str) -> Option<usize> {
    let (base, index) = split_index(prefix).map_or((prefix, 0), |(base, i)| (base, i));
    let mut out = None;
    walk_under(record, base, None, Span::Before(&mut out));
    out.map(|(seen, width)| seen + index * width)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::cells::ByteView;
    use crate::memory::session::{Byte, Dword, Float, Word};

    /// A record of fixed layout with arrays and a nested record, for the byte view's paths.
    #[repr(C)]
    #[derive(rc2k_derive::Cells)]
    struct Pair {
        speed: Float,
        gear: Dword,
    }

    #[repr(C)]
    #[derive(rc2k_derive::Cells)]
    struct Fixed {
        head: [Dword; 3],
        pairs: [Pair; 3],
        ratio: Float,
        tail: [Float; 4],
    }

    fn fixed() -> Fixed {
        Fixed {
            head: [Dword::new(1), Dword::new(2), Dword::new(3)],
            pairs: core::array::from_fn(|k| Pair {
                speed: Float::new(k as f32),
                gear: Dword::new(k as u32),
            }),
            ratio: Float::new(0.5),
            tail: core::array::from_fn(|k| Float::new(k as f32 + 0.25)),
        }
    }

    /// Every path `field_path` names in a record of fixed layout reaches the cell `cell_run`
    /// finds at the same byte.
    fn check_against_the_runs<T: ByteView>(record: &T) {
        let base = std::ptr::from_ref(record) as usize;
        let mut checked = 0;
        for offset in 0..size_of::<T>() {
            let mut path = String::new();
            record.field_path(offset, &mut path);
            let (start, run) = record.cell_run(offset);
            let w = run.width();
            let cell = run.first() + (offset - start) / w * w;
            let found = first_addr(record, &path)
                .unwrap_or_else(|| panic!("{path} (byte {offset:#x}) resolves"));
            // A path without an index names the run's first cell.
            if split_index(&path).is_some() {
                assert_eq!(found, cell, "{path} (byte {offset:#x})");
            } else {
                assert!(found <= cell && cell < found + run.len(), "{path}");
            }
            assert!(found >= base && found < base + size_of::<T>());
            checked += 1;
        }
        assert_eq!(checked, size_of::<T>());
    }

    #[test]
    fn the_paths_field_path_names_reach_the_cells_cell_run_finds() {
        check_against_the_runs(&fixed());
    }

    #[test]
    fn a_fixed_records_cells_round_trip_by_path() {
        let blocks = fixed();
        let mut paths = Vec::new();
        for offset in (0..size_of::<Fixed>()).step_by(4) {
            let mut path = String::new();
            blocks.field_path(offset, &mut path);
            paths.push(path);
        }
        paths.dedup();
        for (n, path) in paths.iter().enumerate() {
            let Some(old) = bytes(&blocks, path) else {
                panic!("{path} resolves");
            };
            let new: Vec<u8> = (0..old.len()).map(|k| (n + k) as u8 ^ 0x5A).collect();
            assert!(set_bytes(&blocks, path, &new), "{path}");
            assert_eq!(bytes(&blocks, path).as_deref(), Some(&new[..]), "{path}");
        }
    }

    #[test]
    fn a_record_without_a_fixed_layout_is_read_and_written_by_path() {
        #[derive(rc2k_derive::Cells)]
        struct Inner {
            speed: Float,
            gear: Byte,
        }
        #[derive(rc2k_derive::Cells)]
        struct Free {
            a: Byte,
            b: Dword,
            c: [Word; 2],
            cars: [Inner; 2],
            inline: Inner,
        }
        let record = Free {
            a: Byte::new(1),
            b: Dword::new(0x1234_5678),
            c: [Word::new(0xAABB), Word::new(0xCCDD)],
            cars: [
                Inner {
                    speed: Float::new(1.5),
                    gear: Byte::new(2),
                },
                Inner {
                    speed: Float::new(-2.0),
                    gear: Byte::new(3),
                },
            ],
            inline: Inner {
                speed: Float::new(9.0),
                gear: Byte::new(4),
            },
        };
        assert_eq!(get_u8(&record, ".a"), Some(1));
        assert_eq!(get_u32(&record, ".b"), Some(0x1234_5678));
        assert_eq!(get_u16(&record, ".c[1]"), Some(0xCCDD));
        assert_eq!(get_f32(&record, ".cars[1].speed"), Some(-2.0));
        assert_eq!(get_u8(&record, ".cars[0].gear"), Some(2));
        // `inline` adds nothing to a path.
        assert_eq!(get_f32(&record, ".speed"), Some(9.0));
        assert_eq!(get_u8(&record, ".gear"), Some(4));
        // The whole array is one piece without an index.
        assert_eq!(bytes(&record, ".c").map(|b| b.len()), Some(4));
        assert_eq!(get_u32(&record, ".nothing"), None);
        assert_eq!(get_u16(&record, ".c[2]"), None);
        assert_eq!(get_u32(&record, ".a"), None);
        assert!(set_f32(&record, ".cars[0].speed", 7.25));
        assert!(set_u16(&record, ".c[0]", 0x0102));
        assert!(set_u8(&record, ".cars[1].gear", 9));
        assert!(!set_u32(&record, ".a", 5));
        assert_eq!(get_f32(&record, ".cars[0].speed"), Some(7.25));
        assert_eq!(get_u16(&record, ".c[0]"), Some(0x0102));
        assert_eq!(get_u16(&record, ".c[1]"), Some(0xCCDD));
        assert_eq!(get_u8(&record, ".cars[1].gear"), Some(9));
        assert_eq!(get_u8(&record, ".a"), Some(1));
        let (a, b) = (
            first_addr(&record, ".c[0]").unwrap(),
            first_addr(&record, ".c[1]").unwrap(),
        );
        assert_eq!(b - a, 2);
    }

    #[test]
    fn a_range_starts_at_a_cell_of_an_array() {
        #[derive(rc2k_derive::Cells)]
        struct Free {
            a: Dword,
            b: [Dword; 3],
            c: Dword,
        }
        let record = Free {
            a: Dword::new(1),
            b: [Dword::new(2), Dword::new(3), Dword::new(4)],
            c: Dword::new(5),
        };
        let words = |bytes: Vec<u8>| -> Vec<u32> {
            bytes
                .chunks(4)
                .map(|w| u32::from_le_bytes(w.try_into().unwrap()))
                .collect()
        };
        assert_eq!(words(field_range(&record, ".b[1]", 12)), [3, 4, 5]);
        assert_eq!(words(field_range(&record, ".b[2]", 4)), [4]);
        let put: Vec<u8> = [7u32, 8].iter().flat_map(|w| w.to_le_bytes()).collect();
        assert!(set_field_range(&record, ".b[2]", &put));
        assert_eq!(
            (
                record.a.get(),
                record.b[1].get(),
                record.b[2].get(),
                record.c.get()
            ),
            (1, 3, 7, 8)
        );
    }
}
