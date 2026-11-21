//! The game's state as the registry's stores hold it: a test that writes the game's
//! variables takes a snapshot of the stores it writes first and puts it back after. Hold
//! `test_support::test_lock` around both. Name only the stores the test writes: putting back a
//! store another test is writing undoes that test's work.

/// A copy of some stores of the registry (of the records they are parts of), to be put back.
pub struct Snapshot(Vec<Box<dyn Fn() + 'static>>);

impl Snapshot {
    /// The stores named `names` (registry names, the first of each; or the scene's plain
    /// stores, `kept_plain`).
    pub fn of(names: &[&str]) -> Self {
        Self(
            names
                .iter()
                .map(|name| kept_plain(name).unwrap_or_else(|| walk(name).keep()))
                .collect(),
        )
    }

    /// The copies put back, store by store.
    pub fn restore(&self) {
        for put_back in &self.0 {
            put_back();
        }
    }
}

/// The scene's plain stores (values the registry does not walk): a copy of the one named `name`,
/// to be put back; `None` for any other name.
fn kept_plain(name: &str) -> Option<Box<dyn Fn() + 'static>> {
    match name {
        // The game's float constants: fixed, nothing to keep.
        "owners.race.course.numbers" => Some(Box::new(|| {})),
        "owners.scene.draw_work" => {
            let copy = super::own::scene().draw_work().clone();
            Some(Box::new(move || {
                *super::own::scene().draw_work_mut() = copy.clone();
            }))
        }
        "owners.race.frame.frame_work" => {
            let copy = *super::own::race().frame.frame_work.borrow();
            Some(Box::new(move || {
                *super::own::race().frame.frame_work.borrow_mut() = copy;
            }))
        }
        "owners.race.frame.shadow_skids" => {
            let copy = super::own::race().frame.shadow_skids.borrow().clone();
            Some(Box::new(move || {
                *super::own::race().frame.shadow_skids.borrow_mut() = copy.clone();
            }))
        }
        "owners.scene.line_and_shake" => {
            let copy = *super::own::scene().line_and_shake();
            Some(Box::new(move || {
                *super::own::scene().line_and_shake_mut() = copy;
            }))
        }
        _ => None,
    }
}

/// A record's cells copied out whole, to be copied back: the typed counterpart of `Snapshot`
/// for a test that writes one record it holds.
struct RecordSnapshot<'a, T: crate::memory::cells::Cells> {
    record: &'a T,
    copy: T,
}

impl Snapshot {
    /// A copy of every cell of `record` (a zeroed record filled by `copy_from`).
    fn of_record<T: crate::memory::cells::Cells>(record: &T) -> RecordSnapshot<'_, T> {
        let copy = crate::memory::cells::zeroed_cells::<T>();
        copy.copy_from(record);
        RecordSnapshot { record, copy }
    }
}

impl<T: crate::memory::cells::Cells> RecordSnapshot<'_, T> {
    /// The cells put back into the record.
    pub fn restore(&self) {
        self.record.copy_from(&self.copy);
    }
}

/// The store named `name`, walkable by field paths.
pub fn walk(name: &str) -> &'static (dyn super::registry::Walk + Sync) {
    // The first entry of each name, by name (the vectors look stores up in every case).
    static FIRST: std::sync::OnceLock<std::collections::HashMap<&'static str, usize>> =
        std::sync::OnceLock::new();
    let listed = super::registry::listed();
    let first = FIRST.get_or_init(|| {
        let mut first = std::collections::HashMap::new();
        for (n, e) in listed.iter().enumerate() {
            first.entry(e.name).or_insert(n);
        }
        first
    });
    let n = *first
        .get(name)
        .unwrap_or_else(|| panic!("no store {name} in the registry"));
    listed[n].store
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_snapshot_puts_a_store_back() {
        let _g = crate::test_support::test_lock();
        let snap = Snapshot::of(&["owners.race.shared.random.state"]);
        let s = walk("owners.race.shared.random.state");
        let before = s.path_bytes("").unwrap();
        assert!(s.set_path_bytes("", &[1, 2, 3, 4]));
        assert_ne!(s.path_bytes("").unwrap(), before);
        snap.restore();
        assert_eq!(s.path_bytes("").unwrap(), before);
    }

    #[test]
    fn a_record_snapshot_puts_a_record_back() {
        use crate::memory::session::{Byte, Dword};
        #[derive(rc2k_derive::Cells)]
        struct Free {
            a: Byte,
            b: [Dword; 2],
        }
        let record = Free {
            a: Byte::new(1),
            b: [Dword::new(2), Dword::new(3)],
        };
        let snap = Snapshot::of_record(&record);
        record.a.set(9);
        record.b[1].set(8);
        snap.restore();
        assert_eq!(record.a.get(), 1);
        assert_eq!(record.b[0].get(), 2);
        assert_eq!(record.b[1].get(), 3);
    }
}
