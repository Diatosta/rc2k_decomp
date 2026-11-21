//! The best times: the six rallies' tables (`RallyTimes`), then the memory the nine other tables
//! are taken on, the two switches and where each table is (`BestTimes`). The nine tables are
//! typed and Rust's own (`saves::best_times`): each pair keeps only the size. `TimeEntry` is a
//! plain record: `saves::options_state` writes and reads its fields in the file's order.

use crate::memory::plain_buffer::PlainBuffer;
use crate::memory::session::PlainDword;

/// Where one table is: its address (0, since the tables are typed) and its size in bytes.
pub struct TablePlace {
    pub address: PlainDword,
    pub size: PlainDword,
}

impl TablePlace {
    const fn new() -> Self {
        Self {
            address: PlainDword::new(0),
            size: PlainDword::new(0),
        }
    }
}

pub struct BestTimes {
    /// The memory list (a `MemoryList`).
    pub memory: crate::memory::PlainList,
    /// 1 once the memory is taken.
    pub allocated: PlainDword,
    /// 1 once the tables hold times (the file's or made up).
    pub loaded: PlainDword,
    /// `BestTimes.tables[0].address` ..: the nine tables, in `rc2k_core::saves::tables`' order.
    pub tables: [TablePlace; 9],
}

impl BestTimes {
    pub const fn new() -> Self {
        BestTimes {
            memory: crate::memory::PlainList::empty(),
            allocated: PlainDword::new(0),
            loaded: PlainDword::new(0),
            tables: [const { TablePlace::new() }; 9],
        }
    }

    pub fn list(&self) -> &crate::memory::PlainList {
        &self.memory
    }
}

/// A best-time entry (0x9C bytes), as the rally and stage tables hold them (and the
/// standings' table A, `menu::standings`). Fields by `rc2k_core::saves::defaults::TimeEntry`
/// and the section screen (readings).
pub struct TimeEntry {
    /// Read by no code.
    pub unread_0: PlainDword,
    /// The player (1 .. 4), 0 for a computer driver.
    pub player: PlainDword,
    /// The nationality (the driver's `nation`).
    pub nation: PlainDword,
    /// The name: a driver's string or a player.
    pub name_id: crate::saves::name_cell::NameIdCell,
    /// The car (the driver's `car`).
    pub car: PlainDword,
    /// The time in a stage's table.
    pub stage_time: PlainDword,
    /// The time in a rally's table.
    pub rally_time: PlainDword,
    /// The group (the driver's class).
    pub group: PlainDword,
    /// The name, NUL-ended.
    pub name: PlainBuffer<{ TimeEntry::NAME_LEN }>,
    /// 1 for a player's time.
    pub is_player: PlainDword,
    /// A figure.
    pub figure: PlainDword,
    /// Read by no code but as the name's tail (a name without a NUL reads on into
    /// it); 0 in the shipped data.
    pub unread_48: [PlainDword; 2],
    /// Read by no code (0xB in a made-up entry).
    pub unread_50: PlainDword,
    /// The two dwords after `unread_50`: read by no code.
    pub unread_54: [PlainDword; 2],
    /// The checkpoints' times.
    pub checkpoints: [PlainDword; 16],
}

impl TimeEntry {
    /// The bytes of the name field.
    pub const NAME_LEN: usize = 0x20;
    /// The entry's length in the files and tables: 39 dwords.
    pub const FILE_LEN: usize = 0x9C;

    /// A made-up time (`saves::defaults`), field by field: the time at `time_at` (`stage_time` or
    /// `rally_time`), the driver's first field as the group, the second as the
    /// nationality, 0xB in `unread_50`, the guessed checkpoints when it has them, the car, the name's
    /// string id and the name's bytes. Refuses a name longer than the field, writing nothing.
    pub fn fill_made_up(
        &self,
        e: &rc2k_core::saves::defaults::TimeEntry,
        time_at: usize,
    ) -> Result<(), crate::errors::BadInput> {
        let time = match time_at {
            0x14 => &self.stage_time,
            0x18 => &self.rally_time,
            _ => {
                return Err(crate::errors::BadInput(
                    "a made-up time written at neither time field",
                ));
            }
        };
        if e.driver.name.len() > Self::NAME_LEN {
            return Err(crate::errors::BadInput(
                "a made-up name longer than the name field",
            ));
        }
        time.set(e.time);
        self.group.set(e.driver.fields[0]);
        self.nation.set(e.driver.fields[1]);
        self.unread_50.set(0xB);
        if let Some(checkpoints) = e.checkpoints {
            for (cell, c) in self.checkpoints.iter().zip(checkpoints) {
                cell.set(c);
            }
        }
        self.car.set(e.driver.fields[2]);
        self.name_id.set(e.driver.name_id);
        for (n, b) in e.driver.name.iter().enumerate() {
            self.name.set(n, *b);
        }
        Ok(())
    }

    /// From the dwords but the name's (31 of them, in order) and the name's bytes.
    const fn new(w: [u32; 31], name: [u8; Self::NAME_LEN]) -> Self {
        let mut checkpoints = [const { PlainDword::new(0) }; 16];
        let mut k = 0;
        while k < 16 {
            checkpoints[k] = PlainDword::new(w[15 + k]);
            k += 1;
        }
        Self {
            unread_0: PlainDword::new(w[0]),
            player: PlainDword::new(w[1]),
            nation: PlainDword::new(w[2]),
            name_id: crate::saves::name_cell::NameIdCell::from_file(w[3]),
            car: PlainDword::new(w[4]),
            stage_time: PlainDword::new(w[5]),
            rally_time: PlainDword::new(w[6]),
            group: PlainDword::new(w[7]),
            name: PlainBuffer::new(name),
            is_player: PlainDword::new(w[8]),
            figure: PlainDword::new(w[9]),
            unread_48: [PlainDword::new(w[10]), PlainDword::new(w[11])],
            unread_50: PlainDword::new(w[12]),
            unread_54: [PlainDword::new(w[13]), PlainDword::new(w[14])],
            checkpoints,
        }
    }
}

/// The six rallies' best times: `RallyTimes.tables[0]`, the six tables' names (addresses of
/// the old layout; `RallyTimes::table_named` makes one real, for
/// `screens::draw_section_best_times` and `saves::default_championship_times`), then the
/// tables, twenty entries each.
pub struct RallyTimes {
    pub tables: [PlainDword; 6],
    pub rallies: [[TimeEntry; 20]; 6],
}

/// What every entry of the six rallies' tables holds at the start: no player, no name, the
/// slowest time in both time fields, group 0x63 and every checkpoint time at -1 (the first
/// 15 dwords are `TimeEntry`'s fields up to `unread_54`, then the 16 checkpoints).
const EMPTY_ENTRY: [u32; 31] = {
    let mut row = [0xffff_ffff; 31];
    let mut k = 0;
    while k < 15 {
        row[k] = 0;
        k += 1;
    }
    row[5] = 0x1ffb8;
    row[6] = 0x1ffb8;
    row[7] = 0x63;
    row
};

impl RallyTimes {
    pub const fn new() -> Self {
        let mut tables = [const { PlainDword::new(0) }; 6];
        let mut n = 0;
        while n < 6 {
            tables[n] =
                PlainDword::new((Self::FIRST_TABLE + n * (20 * TimeEntry::FILE_LEN)) as u32);
            n += 1;
        }
        RallyTimes {
            tables,
            rallies: [const { [const { TimeEntry::new(EMPTY_ENTRY, [0; 0x20]) }; 20] }; 6],
        }
    }
}

impl RallyTimes {
    /// The number of the table the name kept for rally `n` names (its own, when the data is
    /// whole).
    pub fn table_named(&self, n: usize) -> Result<usize, crate::errors::BadInput> {
        let name = self.tables.get(n).ok_or(crate::errors::BadInput(
            "a rally past the six best-times tables",
        ))?;
        let name = name.get() as usize;
        let table_len = 20 * TimeEntry::FILE_LEN;
        if !(Self::START..Self::END).contains(&name) || name < Self::FIRST_TABLE {
            return Err(crate::errors::BadInput(
                "a best-times table off the rallies' tables",
            ));
        }
        let at = name - Self::FIRST_TABLE;
        if !at.is_multiple_of(table_len) || at / table_len >= self.rallies.len() {
            return Err(crate::errors::BadInput(
                "a best-times table off the rallies' tables",
            ));
        }
        Ok(at / table_len)
    }

    /// Rally `n`'s table, when the name kept for it (`table_named`) is that table.
    pub fn table(&self, n: usize) -> Result<&[TimeEntry; 20], crate::errors::BadInput> {
        let named = self.table_named(n)?;
        let table = self.rallies.get(n).ok_or(crate::errors::BadInput(
            "a rally past the six best-times tables",
        ))?;
        if named != n {
            return Err(crate::errors::BadInput(
                "a rally's best-times name not at its own table",
            ));
        }
        Ok(table)
    }

    /// Where the first rally's table is in the old layout (the six names follow the piece's start).
    const FIRST_TABLE: usize = 0x742CB0;
    /// The old address of the piece's first byte.
    pub const START: usize = 0x742C98;
    /// The old address of its end.
    pub const END: usize = 0x7475D0;
}

#[cfg(test)]
impl TimeEntry {
    /// The entry's bytes in the files' order.
    fn file_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::FILE_LEN);
        for field in [
            &self.unread_0,
            &self.player,
            &self.nation,
            self.name_id.file_dword(),
            &self.car,
            &self.stage_time,
            &self.rally_time,
            &self.group,
        ] {
            out.extend(field.get().to_le_bytes());
        }
        out.extend(self.name.read());
        for field in [&self.is_player, &self.figure]
            .into_iter()
            .chain(&self.unread_48)
            .chain([&self.unread_50])
            .chain(&self.unread_54)
            .chain(&self.checkpoints)
        {
            out.extend(field.get().to_le_bytes());
        }
        out
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn rally_times_names_resolve() {
        let rally_times = &crate::test_support::own::saves().rally_times;
        // The six names are the tables' places and resolve to them.
        let first = rally_times.tables[0].get();
        for n in 0..6 {
            let table = 20 * 0x9C * n;
            assert_eq!(rally_times.tables[n].get() as usize, first as usize + table);
            assert_eq!(rally_times.table_named(n), Ok(n));
        }
        assert!(rally_times.table_named(6).is_err());
    }
}

#[cfg(test)]
mod table_named_tests {

    #[test]
    fn table_named_resolves_the_six_names_and_no_more() {
        let _g = crate::test_support::test_lock();
        let times = &crate::test_support::own::saves().rally_times;
        for n in 0..8 {
            assert_eq!(times.table_named(n).is_ok(), n < 6, "rally {n}");
            if n < 6 {
                assert!(times.table(n).is_ok(), "rally {n}");
            }
        }
    }
}

#[cfg(test)]
mod fill_made_up_tests {
    use super::*;

    /// A made-up time written by fields is the entry the core writes by byte position, in a stage's
    /// table (`stage_time`) and a rally's (`rally_time`); a name longer than the field and another place for
    /// the time are refused, writing nothing.
    #[test]
    fn a_made_up_time_is_the_cores_bytes() {
        let _g = crate::test_support::test_lock();
        let made = rc2k_core::saves::defaults::TimeEntry {
            time: 0x1234,
            driver: rc2k_core::saves::defaults::Driver {
                fields: [7, 8, 9],
                name_id: rc2k_core::saves::name_id::NameId::String(0x5A6),
                name: b"Somebody Else".to_vec(),
            },
            checkpoints: Some(std::array::from_fn(|n| 100 + n as u32)),
        };
        for time_at in [0x14, 0x18] {
            let entry = TimeEntry::new([0; 31], [0; 0x20]);
            let mut want = entry.file_bytes();
            made.write(&mut want, time_at);
            entry.fill_made_up(&made, time_at).unwrap();
            let got = entry.file_bytes();
            assert!(got == want, "time at {time_at:#x}");
        }
        let entry = TimeEntry::new([0; 31], [0; 0x20]);
        assert!(entry.fill_made_up(&made, 0x1C).is_err());
        let mut long = made.clone();
        long.driver.name = vec![b'x'; 0x21];
        assert!(entry.fill_made_up(&long, 0x14).is_err());
        assert_eq!(entry.stage_time.get(), 0);
    }
}

#[cfg(test)]
mod image_bytes_tests {
    use super::*;

    fn dump(times: &RallyTimes, bt: &BestTimes) -> Vec<u8> {
        let mut out = Vec::new();
        for t in &times.tables {
            out.extend(t.get().to_le_bytes());
        }
        for rally in &times.rallies {
            for entry in rally {
                out.extend(entry.file_bytes());
            }
        }
        out.extend(bt.allocated.get().to_le_bytes());
        out.extend(bt.loaded.get().to_le_bytes());
        for t in &bt.tables {
            out.extend(t.address.get().to_le_bytes());
            out.extend(t.size.get().to_le_bytes());
        }
        out
    }

    /// The record's whole bytes, as a checksum: a wrong dword in a table shows here.
    #[test]
    fn the_best_times_keep_the_bytes_of_the_image() {
        let bytes = dump(&RallyTimes::new(), &BestTimes::new());
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in &bytes {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
        }
        assert_eq!(bytes.len(), 18824);
        assert_eq!(hash, 0x547531a5b118e289);
    }
}
