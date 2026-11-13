//! The game's state as the registry's stores hold it: a test that writes the game's
//! variables takes a snapshot of the stores it writes first and puts it back after. Hold
//! `test_support::test_lock` around both. Name only the stores the test writes: putting back a
//! store another test is writing undoes that test's work.

/// The bytes of some stores of the registry.
pub struct Snapshot(Vec<(&'static (dyn crate::cells::Store + Sync), Vec<u8>)>);

impl Snapshot {
    /// The stores named `names` (registry names, the first of each).
    pub fn of(names: &[&str]) -> Self {
        Self(
            names
                .iter()
                .map(|name| {
                    let s = store(name);
                    let mut bytes = vec![0; s.size()];
                    s.load(0, &mut bytes).expect("a store's own bytes");
                    (s, bytes)
                })
                .collect(),
        )
    }

    /// The bytes put back, store by store.
    pub fn restore(&self) {
        for (s, bytes) in &self.0 {
            s.store(0, bytes).expect("a store's own bytes");
        }
    }
}

/// The registry's store named `name` (the first of that name): what a test reads or
/// writes by offset (`Store::load`, `Store::store`) where it named an address.
pub fn store(name: &str) -> &'static (dyn crate::cells::Store + Sync) {
    walk(name)
}

/// The same store, walkable by cell runs and field paths.
pub fn walk(name: &str) -> &'static (dyn super::registry::Walk + Sync) {
    super::registry::listed()
        .iter()
        .find(|e| e.name == name)
        .unwrap_or_else(|| panic!("no store {name} in the registry"))
        .store
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_snapshot_puts_a_store_back() {
        let _g = crate::test_support::test_lock();
        let snap = Snapshot::of(&["math.next_random.STATE"]);
        let s = store("math.next_random.STATE");
        let mut before = [0; 4];
        s.load(0, &mut before).unwrap();
        s.store(0, &[1, 2, 3, 4]).unwrap();
        snap.restore();
        let mut after = [0; 4];
        s.load(0, &mut after).unwrap();
        assert_eq!(before, after);
    }
}
