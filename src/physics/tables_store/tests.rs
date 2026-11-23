use super::PhysicsTables;
use crate::memory::cells::CellVisitor;
use crate::memory::plain_record::PlainRecord;

/// Every leaf of a walk, in order.
#[derive(Default)]
struct Bytes(Vec<u8>);

impl CellVisitor for Bytes {
    fn enter(&mut self, _: &'static str) {}
    fn index(&mut self, _: usize) {}
    fn leave(&mut self) {}
    fn leaf(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
}

/// The tables' whole bytes, as a checksum: a wrong dword in a row shows here.
#[test]
fn the_physics_tables_keep_the_bytes_of_the_image() {
    let mut walk = Bytes::default();
    PhysicsTables::new().visit(&mut walk);
    let bytes = walk.0;
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in &bytes {
        hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
    }
    assert_eq!(bytes.len(), 2236);
    assert_eq!(hash, 4_048_912_719_414_518_593);
}

#[test]
fn the_ground_records_name_whole_tables() {
    let shared = &crate::test_support::own::race().shared;
    let tables = shared.physics_tables();
    for n in 0..16u32 {
        let slot = tables.surface_table_index(n).unwrap();
        assert!(slot.is_multiple_of(13) && slot < 104);
    }
    assert!(tables.surface_table_index(16).is_err());
}

/// Every call a walk makes, as text.
#[derive(Default)]
struct Trace(Vec<String>);

impl CellVisitor for Trace {
    fn enter(&mut self, name: &'static str) {
        self.0.push(format!("enter {name}"));
    }
    fn index(&mut self, i: usize) {
        self.0.push(format!("index {i}"));
    }
    fn leave(&mut self) {
        self.0.push("leave".into());
    }
    fn leaf(&mut self, bytes: &[u8]) {
        self.0.push(format!("leaf {bytes:?}"));
    }
}

/// The plain record walks as the record of cells it replaced did: the same field names, indices
/// and bytes, so what reads the tables by path sees no difference.
#[test]
fn the_plain_tables_walk_as_the_cells_did() {
    use crate::memory::cells::{Cells, ZeroCells};
    let old = super::old_cells::OldPhysicsTables::ZERO;
    let mut cells = Trace::default();
    old.visit(&mut cells);
    let mut plain = Trace::default();
    PhysicsTables::default().visit(&mut plain);
    assert!(cells.0.len() > 100);
    assert_eq!(plain.0, cells.0);
}
