//! The championship's drivers: the 180 computer drivers' records (0x94 bytes) and the -1 that
//! ends them for the walks (`menu::standings::walked`). A driver's name entry is its string
//! `NAME_STRINGS + entry`.
//! Plain fields behind one `RefCell` (the game's thread alone reads and writes them, each borrow
//! lasting one statement). The savegame holds the records as bytes (`saves::gather_savegame`,
//! `load_savegame`): `DriverTable::to_bytes` / `load_bytes` give and take the fields' dwords in
//! declaration order, which the test pins to the file's offsets.
//! The fields' meanings are readings (`class`, `nation` and `car`: the time entries and the
//! section screen say class, nationality and car).

use crate::memory::session::GameThread;
use std::cell::{Ref, RefCell, RefMut};

/// How many drivers there are.
pub const DRIVER_COUNT: usize = 180;

/// A driver's record (0x94 bytes).
#[derive(Clone, Copy)]
pub struct Driver {
    /// The name entry, also the computer team's number (`menu::find_cpu_team`).
    pub name_entry: u32,
    /// 1 in all 180 shipped records; read by no code (saved with the record).
    pub reserved_04: u32,
    /// The class (the `group` of a time entry).
    pub class: u32,
    /// The stage's time (`menu::calc_cpu_times`; a time entry's time).
    pub time: u32,
    /// The event's total time, each stage's added.
    pub total: u32,
    /// The nationality (a time entry's `nation`, the flag's sprite; `var\name.cfg`'s first
    /// number).
    pub nation: u32,
    /// The car (a time entry's `car`; `name.cfg`'s second number).
    pub car: u32,
    /// 0 in all 180 shipped records; read by no code (saved with the record).
    pub reserved_1c: u32,
    /// 0 in all 180 shipped records and put back to 0 at an event's start (`begin`); read by
    /// no code (saved with the record).
    pub unread_zero_20: u32,
    /// 1 while in the event (`IN_EVENT`), 4 once dropped out
    /// (`rc2k_core::menu::event_races::DROPPED`).
    pub in_event: u32,
    /// 0 in all 180 shipped records; read by no code (saved with the record).
    pub reserved_28: u32,
    /// Twelve figures set to 100 at an event's start (the car's condition).
    pub condition: [u32; 12],
    /// The result slots: six sections' overall points and their total, then six
    /// sections' stage points and their total (`menu::clear_result_slots`,
    /// `menu::award_overall_points`, `menu::award_stage_points`).
    pub results: [u32; 14],
}

/// A driver's record in the savegame, in bytes.
pub(super) const DRIVER_LEN: usize = 0x94;

/// The dwords of a record.
const WORDS: usize = DRIVER_LEN / 4;

impl Driver {
    /// Driver `index` of the championship: the fields the 180 shipped records share are
    /// filled in (see `DRIVER_ROWS` for the ones that differ).
    const fn new(index: u32, [class, nation, car]: [u32; 3]) -> Self {
        Self {
            name_entry: index,
            reserved_04: 1,
            class,
            time: 0,
            total: 0,
            nation,
            car,
            reserved_1c: 0,
            unread_zero_20: 0,
            in_event: IN_EVENT,
            reserved_28: 0,
            condition: [100; 12],
            // Six sections' overall points and their total (the last of them 0), then the
            // same for the stage points.
            results: [
                0xFFFF_FFFF,
                0xFFFF_FFFF,
                0xFFFF_FFFF,
                0xFFFF_FFFF,
                0xFFFF_FFFF,
                0xFFFF_FFFF,
                0,
                0xFFFF_FFFF,
                0xFFFF_FFFF,
                0xFFFF_FFFF,
                0xFFFF_FFFF,
                0xFFFF_FFFF,
                0xFFFF_FFFF,
                0,
            ],
        }
    }

    /// The record's dwords in the file's order (the fields' declaration order).
    fn words(&self) -> [u32; WORDS] {
        let mut out = [0u32; WORDS];
        out[..11].copy_from_slice(&[
            self.name_entry,
            self.reserved_04,
            self.class,
            self.time,
            self.total,
            self.nation,
            self.car,
            self.reserved_1c,
            self.unread_zero_20,
            self.in_event,
            self.reserved_28,
        ]);
        out[11..23].copy_from_slice(&self.condition);
        out[23..].copy_from_slice(&self.results);
        out
    }

    /// The record `words` (the file's order) hold.
    fn from_words(w: &[u32; WORDS]) -> Self {
        Self {
            name_entry: w[0],
            reserved_04: w[1],
            class: w[2],
            time: w[3],
            total: w[4],
            nation: w[5],
            car: w[6],
            reserved_1c: w[7],
            unread_zero_20: w[8],
            in_event: w[9],
            reserved_28: w[10],
            condition: std::array::from_fn(|k| w[11 + k]),
            results: std::array::from_fn(|k| w[23 + k]),
        }
    }
}

/// A driver's `in_event` while in the event.
pub(super) const IN_EVENT: u32 = 1;

impl Driver {
    /// Back in the event with no times (`menu::reset_drivers`).
    pub fn reset(&mut self) {
        self.in_event = IN_EVENT;
        self.time = 0;
        self.total = 0;
    }

    /// An event's start (`menu::begin_event`, `menu::reset_single_event`): no times, the
    /// unread field 0, in the event.
    pub fn begin(&mut self) {
        self.reset();
        self.unread_zero_20 = 0;
    }

    /// The twelve condition figures back at 100 (`menu::reset_driver_condition`).
    pub(super) fn reset_condition(&mut self) {
        self.condition = [100; 12];
    }

    /// The result slots emptied (`menu::clear_result_slots`).
    pub fn clear_results(&mut self) {
        for (k, slot) in self.results.iter_mut().enumerate() {
            *slot = rc2k_core::menu::event::empty_result_slot(k);
        }
    }

    /// The overall points of section `section` (0 .. 5; `menu::award_overall_points`).
    pub fn overall(&mut self, section: usize) -> &mut u32 {
        &mut self.results[section]
    }

    /// The total of the overall points.
    pub(super) fn overall_total(&mut self) -> &mut u32 {
        &mut self.results[6]
    }

    /// The stage points of section `section` (0 .. 5; `menu::award_stage_points`).
    pub fn stage(&mut self, section: usize) -> &mut u32 {
        &mut self.results[7 + section]
    }

    /// The total of the stage points.
    pub(super) fn stage_total(&mut self) -> &mut u32 {
        &mut self.results[13]
    }
}

/// The records and the walks' end.
pub struct DriverTables {
    pub drivers: [Driver; DRIVER_COUNT],
    /// -1, the walks' end.
    pub end: u32,
}

/// The drivers' records and their end mark, behind one `RefCell`.
pub struct DriverTable(GameThread<RefCell<DriverTables>>);

/// The columns of the 180 driver records that differ from driver to driver, one row a
/// driver (the row number is the driver's number and its name entry): class, nation
/// and car. Every other field is the same in all 180 (`Driver::new`).
#[rustfmt::skip]
const DRIVER_ROWS: [[u32; 3]; 180] = [
    [0, 0, 4], // 0
    [0, 25, 7], // 1
    [0, 3, 2], // 2
    [0, 5, 4], // 3
    [0, 21, 4], // 4
    [0, 37, 22], // 5
    [0, 6, 14], // 6
    [0, 7, 2], // 7
    [0, 8, 2], // 8
    [0, 9, 14], // 9
    [0, 10, 2], // 10
    [0, 11, 14], // 11
    [0, 13, 10], // 12
    [0, 5, 14], // 13
    [0, 12, 2], // 14
    [0, 13, 7], // 15
    [0, 18, 22], // 16
    [0, 19, 2], // 17
    [0, 20, 7], // 18
    [0, 2, 2], // 19
    [0, 22, 2], // 20
    [0, 23, 22], // 21
    [0, 16, 14], // 22
    [0, 24, 2], // 23
    [0, 26, 10], // 24
    [0, 27, 4], // 25
    [0, 2, 4], // 26
    [0, 6, 22], // 27
    [0, 28, 2], // 28
    [0, 29, 4], // 29
    [0, 30, 7], // 30
    [0, 31, 4], // 31
    [0, 32, 7], // 32
    [0, 14, 14], // 33
    [0, 33, 4], // 34
    [0, 34, 10], // 35
    [0, 35, 14], // 36
    [0, 36, 14], // 37
    [0, 17, 22], // 38
    [0, 4, 22], // 39
    [0, 38, 14], // 40
    [0, 39, 22], // 41
    [0, 0, 2], // 42
    [0, 1, 14], // 43
    [0, 3, 2], // 44
    [0, 15, 10], // 45
    [0, 3, 14], // 46
    [0, 4, 4], // 47
    [0, 17, 4], // 48
    [0, 6, 22], // 49
    [0, 8, 4], // 50
    [0, 9, 7], // 51
    [0, 10, 2], // 52
    [0, 11, 14], // 53
    [0, 33, 10], // 54
    [0, 12, 22], // 55
    [0, 15, 14], // 56
    [0, 16, 10], // 57
    [0, 18, 14], // 58
    [0, 19, 2], // 59
    [0, 20, 22], // 60
    [0, 21, 2], // 61
    [0, 22, 7], // 62
    [0, 14, 7], // 63
    [0, 23, 7], // 64
    [0, 24, 4], // 65
    [0, 7, 14], // 66
    [0, 25, 10], // 67
    [0, 27, 14], // 68
    [0, 28, 14], // 69
    [0, 29, 10], // 70
    [0, 30, 22], // 71
    [0, 31, 22], // 72
    [0, 32, 7], // 73
    [0, 1, 22], // 74
    [0, 34, 22], // 75
    [0, 9, 7], // 76
    [0, 35, 4], // 77
    [0, 36, 10], // 78
    [0, 38, 7], // 79
    [0, 39, 14], // 80
    [0, 0, 4], // 81
    [0, 1, 4], // 82
    [0, 26, 7], // 83
    [0, 37, 10], // 84
    [0, 2, 14], // 85
    [0, 4, 7], // 86
    [0, 7, 14], // 87
    [0, 5, 22], // 88
    [0, 8, 7], // 89
    [1, 0, 1], // 90
    [1, 13, 3], // 91
    [1, 15, 3], // 92
    [1, 23, 15], // 93
    [1, 12, 8], // 94
    [1, 14, 0], // 95
    [1, 16, 17], // 96
    [1, 33, 17], // 97
    [1, 18, 17], // 98
    [1, 19, 0], // 99
    [1, 0, 9], // 100
    [1, 37, 16], // 101
    [1, 0, 16], // 102
    [1, 12, 1], // 103
    [1, 24, 1], // 104
    [1, 25, 17], // 105
    [1, 26, 0], // 106
    [1, 27, 0], // 107
    [1, 0, 3], // 108
    [1, 29, 16], // 109
    [1, 30, 8], // 110
    [1, 17, 20], // 111
    [1, 31, 8], // 112
    [1, 32, 8], // 113
    [1, 34, 15], // 114
    [1, 11, 20], // 115
    [1, 35, 16], // 116
    [1, 0, 0], // 117
    [1, 36, 3], // 118
    [1, 37, 8], // 119
    [1, 38, 20], // 120
    [1, 0, 1], // 121
    [1, 11, 17], // 122
    [1, 0, 17], // 123
    [1, 14, 15], // 124
    [1, 0, 17], // 125
    [1, 0, 17], // 126
    [1, 0, 16], // 127
    [1, 0, 1], // 128
    [1, 8, 3], // 129
    [1, 9, 9], // 130
    [1, 10, 9], // 131
    [1, 11, 0], // 132
    [1, 0, 15], // 133
    [1, 0, 17], // 134
    [1, 0, 1], // 135
    [1, 16, 17], // 136
    [1, 0, 0], // 137
    [1, 18, 8], // 138
    [1, 19, 3], // 139
    [2, 0, 12], // 140
    [2, 0, 12], // 141
    [2, 0, 5], // 142
    [2, 0, 19], // 143
    [2, 24, 11], // 144
    [2, 25, 11], // 145
    [2, 26, 5], // 146
    [2, 27, 11], // 147
    [2, 28, 5], // 148
    [2, 29, 5], // 149
    [2, 30, 12], // 150
    [2, 31, 12], // 151
    [2, 32, 19], // 152
    [2, 33, 5], // 153
    [2, 34, 5], // 154
    [2, 35, 11], // 155
    [2, 36, 5], // 156
    [2, 0, 5], // 157
    [2, 0, 12], // 158
    [2, 0, 11], // 159
    [2, 0, 11], // 160
    [2, 1, 5], // 161
    [2, 37, 11], // 162
    [2, 0, 19], // 163
    [2, 4, 19], // 164
    [2, 5, 5], // 165
    [2, 37, 11], // 166
    [2, 0, 12], // 167
    [2, 0, 5], // 168
    [2, 13, 19], // 169
    [3, 10, 13], // 170
    [3, 11, 21], // 171
    [3, 12, 21], // 172
    [3, 13, 21], // 173
    [3, 15, 13], // 174
    [3, 14, 13], // 175
    [3, 16, 21], // 176
    [3, 17, 13], // 177
    [3, 18, 13], // 178
    [3, 19, 21], // 179
];

impl DriverTable {
    pub const fn new() -> Self {
        let mut drivers = [Driver::new(0, [0, 0, 0]); DRIVER_COUNT];
        let mut k = 0;
        while k < DRIVER_COUNT {
            drivers[k] = Driver::new(k as u32, DRIVER_ROWS[k]);
            k += 1;
        }
        DriverTable(GameThread::new(RefCell::new(DriverTables {
            drivers,
            end: 0xFFFF_FFFF,
        })))
    }

    /// The tables, to read: no borrow outlives the statement that reads them.
    pub fn read(&self) -> Ref<'_, DriverTables> {
        self.0.get().borrow()
    }

    /// The tables, to write.
    pub fn write(&self) -> RefMut<'_, DriverTables> {
        self.0.get().borrow_mut()
    }

    /// `f` on each of the first `count` drivers.
    pub fn each(&self, count: usize, f: fn(&mut Driver)) {
        self.write().drivers[..count].iter_mut().for_each(f);
    }

    /// The bytes of the 180 records as the savegame holds them: each record's dwords in the
    /// file's order (`the_records_are_the_files_bytes` holds it to the offsets).
    pub fn to_bytes(&self) -> Vec<u8> {
        self.read()
            .drivers
            .iter()
            .flat_map(|d| d.words().into_iter().flat_map(u32::to_le_bytes))
            .collect()
    }

    /// `to_bytes`' inverse: `bytes` (the 180 records, else refused, writing nothing) set into
    /// the records.
    pub fn load_bytes(&self, bytes: &[u8]) -> Result<(), crate::errors::BadInput> {
        if bytes.len() != DRIVER_COUNT * DRIVER_LEN {
            return Err(crate::errors::BadInput(
                "saved drivers of another length than the table",
            ));
        }
        let mut tables = self.write();
        let (chunks, _) = bytes.as_chunks::<DRIVER_LEN>();
        for (driver, chunk) in tables.drivers.iter_mut().zip(chunks) {
            let (words, _) = chunk.as_chunks::<4>();
            let words: [u32; WORDS] = std::array::from_fn(|k| u32::from_le_bytes(words[k]));
            *driver = Driver::from_words(&words);
        }
        Ok(())
    }

    /// Drivers `a` and `b` changed places (`menu::shuffle_drivers`).
    pub fn swap(&self, a: usize, b: usize) {
        self.write().drivers.swap(a, b);
    }
}

/// The string id of the first name entry; entry `n` is `NAME_STRINGS + n`.
const NAME_STRINGS: u32 = 0x5A5;

/// The string id of name entry `entry` (a driver's first dword). An entry past the 180 is
/// refused (a 1999 bug fixed: it read on past them).
pub fn name_string(entry: u32) -> Result<u32, crate::errors::BadInput> {
    if (entry as usize) < DRIVER_COUNT {
        Ok(NAME_STRINGS + entry)
    } else {
        Err(crate::errors::BadInput(
            "a driver's name entry past the 180",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Record `index` as the savegame has it, written by the file's offsets (the layout the
    /// 1999 game kept in memory), not through the fields.
    fn file_record(index: u32) -> [u8; DRIVER_LEN] {
        let [class, nation, car] = DRIVER_ROWS[index as usize];
        let mut b = [0u8; DRIVER_LEN];
        let mut put = |at: usize, v: u32| b[at..at + 4].copy_from_slice(&v.to_le_bytes());
        put(0, index);
        put(4, 1);
        put(8, class);
        put(0x14, nation);
        put(0x18, car);
        put(0x24, IN_EVENT);
        for k in 0..12 {
            put(0x2C + 4 * k, 100);
        }
        for k in 0..14 {
            put(0x5C + 4 * k, if k % 7 == 6 { 0 } else { u32::MAX });
        }
        b
    }

    /// The records' bytes are the file's, record by record, and read back the same.
    #[test]
    fn the_records_are_the_files_bytes() {
        let table = DriverTable::new();
        let expected: Vec<u8> = (0..DRIVER_COUNT as u32).flat_map(file_record).collect();
        let bytes = table.to_bytes();
        assert_eq!(bytes.len(), DRIVER_COUNT * DRIVER_LEN);
        assert!(
            bytes == expected,
            "the records' bytes differ from the file's"
        );
        // Another table takes them and gives them back; a wrong length writes nothing.
        let other = DriverTable::new();
        let changed: Vec<u8> = (0..bytes.len()).map(|n| (n * 7 + 3) as u8).collect();
        other.load_bytes(&changed).unwrap();
        assert!(other.to_bytes() == changed);
        assert!(other.load_bytes(&changed[1..]).is_err());
        assert!(other.to_bytes() == changed);
    }

    /// Two records changing places is the bytes of both swapped.
    #[test]
    fn a_swap_changes_places() {
        let table = DriverTable::new();
        let before = table.to_bytes();
        table.swap(3, 100);
        let after = table.to_bytes();
        let record = |bytes: &[u8], k: usize| bytes[k * DRIVER_LEN..(k + 1) * DRIVER_LEN].to_vec();
        assert_eq!(record(&after, 3), record(&before, 100));
        assert_eq!(record(&after, 100), record(&before, 3));
        table.swap(7, 7);
        assert!(table.to_bytes() == after);
    }
}
