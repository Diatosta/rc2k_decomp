//! The championship's drivers, out of RAL.EXE's image (IDA 0x73B674 .. 0x742C98): `g_drivers`, the 180 computer drivers' records (0x94 bytes, IDA's `Driver`), the
//! -1 that ends them for the walks (`menu::standings::drivers`), and `g_driverNames`, the
//! 180 name entries (0x14 bytes, IDA's `DriverName`) a driver's first dword picks.
//! `#[repr(C)]`, the image's layout. The event code
//! works on the records by field (`menu::standings::drivers` walks them).
//! The fields' meanings are readings (IDA's `Driver` names +8, +0x14 and +0x18
//! `nationality`, `car` and `flag`: the time entries and the section screen say class,
//! nationality and car).

use crate::menu::owner::Menus;
use crate::session::PlainDword;

/// A driver's record (0x94 bytes).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Driver {
    /// +0: the name entry, also the computer team's number (`menu::find_cpu_team`).
    pub name_entry: PlainDword,
    pub field_4: PlainDword,
    /// +8: the class (a time entry's group, +0x1C).
    pub class: PlainDword,
    /// +0xC: the stage's time (`menu::calc_cpu_times`; a time entry's time).
    pub time: PlainDword,
    /// +0x10: the event's total time, each stage's added.
    pub total: PlainDword,
    /// +0x14: the nationality (a time entry's +8, the flag's sprite; `var\name.cfg`'s first
    /// number).
    pub nation: PlainDword,
    /// +0x18: the car (a time entry's +0x10; `name.cfg`'s second number).
    pub car: PlainDword,
    pub field_1c: PlainDword,
    /// +0x20: 0 in all 180 shipped records and put back to 0 at an event's start (`begin`); read by no code (saved with the record).
    pub unread_zero_20: PlainDword,
    /// +0x24: 1 while in the event (`IN_EVENT`), 4 once dropped out
    /// (`rc2k_core::menu::event_races::DROPPED`).
    pub in_event: PlainDword,
    pub field_28: PlainDword,
    /// +0x2C: twelve figures set to 100 at an event's start (the car's condition).
    pub condition: [PlainDword; 12],
    /// +0x5C: the result slots: six sections' overall points and their total, then six
    /// sections' stage points and their total (`menu::clear_result_slots`,
    /// `menu::award_overall_points`, `menu::award_stage_points`).
    pub results: [PlainDword; 14],
}

impl Driver {
    /// Driver `index` of the championship: the fields the 180 shipped records share are
    /// filled in (see `DRIVER_ROWS` for the ones that differ).
    const fn new(index: u32, [class, nation, car]: [u32; 3]) -> Self {
        Self {
            name_entry: PlainDword::new(index),
            field_4: PlainDword::new(1),
            class: PlainDword::new(class),
            time: PlainDword::new(0),
            total: PlainDword::new(0),
            nation: PlainDword::new(nation),
            car: PlainDword::new(car),
            field_1c: PlainDword::new(0),
            unread_zero_20: PlainDword::new(0),
            in_event: PlainDword::new(IN_EVENT),
            field_28: PlainDword::new(0),
            condition: [const { PlainDword::new(100) }; 12],
            // Six sections' overall points and their total (the last of them 0), then the
            // same for the stage points.
            results: [
                const { PlainDword::new(0xFFFF_FFFF) },
                const { PlainDword::new(0xFFFF_FFFF) },
                const { PlainDword::new(0xFFFF_FFFF) },
                const { PlainDword::new(0xFFFF_FFFF) },
                const { PlainDword::new(0xFFFF_FFFF) },
                const { PlainDword::new(0xFFFF_FFFF) },
                const { PlainDword::new(0) },
                const { PlainDword::new(0xFFFF_FFFF) },
                const { PlainDword::new(0xFFFF_FFFF) },
                const { PlainDword::new(0xFFFF_FFFF) },
                const { PlainDword::new(0xFFFF_FFFF) },
                const { PlainDword::new(0xFFFF_FFFF) },
                const { PlainDword::new(0xFFFF_FFFF) },
                const { PlainDword::new(0) },
            ],
        }
    }
}

const _: () = assert!(core::mem::size_of::<Driver>() == 0x94);

/// A driver's +0x24 while in the event.
pub const IN_EVENT: u32 = 1;

impl Driver {
    /// Back in the event with no times (`menu::reset_drivers`).
    pub fn reset(&self) {
        self.in_event.set(IN_EVENT);
        self.time.set(0);
        self.total.set(0);
    }

    /// An event's start (`menu::begin_event`, `menu::reset_single_event`): no times, +0x20
    /// 0, in the event.
    pub fn begin(&self) {
        self.reset();
        self.unread_zero_20.set(0);
    }

    /// The twelve condition figures back at 100 (`menu::reset_driver_condition`).
    pub fn reset_condition(&self) {
        for c in &self.condition {
            c.set(100);
        }
    }

    /// The result slots emptied (`menu::clear_result_slots`).
    pub fn clear_results(&self) {
        for (k, slot) in self.results.iter().enumerate() {
            slot.set(rc2k_core::menu::event::empty_result_slot(k));
        }
    }

    /// The overall points of section `section` (0 .. 5; `menu::award_overall_points`).
    pub fn overall(&self, section: usize) -> &PlainDword {
        &self.results[section]
    }

    /// The total of the overall points.
    pub fn overall_total(&self) -> &PlainDword {
        &self.results[6]
    }

    /// The stage points of section `section` (0 .. 5; `menu::award_stage_points`).
    pub fn stage(&self, section: usize) -> &PlainDword {
        &self.results[7 + section]
    }

    /// The total of the stage points.
    pub fn stage_total(&self) -> &PlainDword {
        &self.results[13]
    }
}

/// A driver's name entry (0x14 bytes): the string's id and four dwords.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct DriverName {
    pub string: PlainDword,
    pub field_4: [PlainDword; 4],
}

impl DriverName {
    /// Name entry `index`: its string id is `0x5A5 + index`, its last dword 100 in all 180;
    /// `NAME_ROWS` holds the three that differ.
    const fn new(index: u32, [a, b, c]: [u32; 3]) -> Self {
        Self {
            string: PlainDword::new(0x5A5 + index),
            field_4: [
                PlainDword::new(a),
                PlainDword::new(b),
                PlainDword::new(c),
                PlainDword::new(100),
            ],
        }
    }
}

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct DriverTable {
    /// `g_drivers` (0x73B674).
    pub drivers: [Driver; 180],
    /// 0x741E84: -1, the walks' end.
    pub end: PlainDword,
    /// `g_driverNames` (0x741E88).
    pub names: [DriverName; 180],
}

/// The columns of the 180 driver records that differ from driver to driver, one row a
/// driver (the row number is the driver's number and its name entry): class (+8), nation
/// (+0x14), car (+0x18). Every other field is the same in all 180 (`Driver::new`).
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

/// The columns of the 180 name entries that differ, one row an entry (the row number is
/// the entry's number): the dwords at +4, +8 and +0xC. The string id (+0) is `0x5A5 +
/// number` and the dword at +0x10 is 100 in all 180 (`DriverName::new`).
#[rustfmt::skip]
const NAME_ROWS: [[u32; 3]; 180] = [
    [0, 9, 10], // 0
    [1, 8, 4], // 1
    [2, 7, 8], // 2
    [3, 6, 5], // 3
    [4, 5, 3], // 4
    [5, 4, 8], // 5
    [6, 3, 7], // 6
    [7, 8, 6], // 7
    [8, 8, 9], // 8
    [9, 3, 6], // 9
    [10, 8, 1], // 10
    [11, 5, 7], // 11
    [12, 7, 6], // 12
    [13, 9, 5], // 13
    [15, 8, 4], // 14
    [14, 7, 7], // 15
    [16, 6, 4], // 16
    [17, 5, 3], // 17
    [18, 4, 5], // 18
    [19, 5, 8], // 19
    [20, 6, 6], // 20
    [21, 4, 7], // 21
    [22, 8, 9], // 22
    [23, 7, 10], // 23
    [24, 6, 10], // 24
    [25, 5, 10], // 25
    [26, 9, 2], // 26
    [27, 8, 1], // 27
    [28, 7, 5], // 28
    [29, 6, 3], // 29
    [30, 5, 5], // 30
    [31, 8, 5], // 31
    [32, 7, 10], // 32
    [33, 9, 4], // 33
    [34, 4, 8], // 34
    [35, 3, 5], // 35
    [36, 5, 3], // 36
    [37, 2, 8], // 37
    [38, 1, 7], // 38
    [39, 9, 6], // 39
    [0, 5, 4], // 40
    [1, 4, 6], // 41
    [2, 4, 10], // 42
    [3, 4, 2], // 43
    [4, 4, 6], // 44
    [5, 4, 5], // 45
    [6, 4, 4], // 46
    [7, 4, 7], // 47
    [8, 4, 4], // 48
    [9, 3, 3], // 49
    [10, 2, 5], // 50
    [11, 1, 8], // 51
    [12, 6, 6], // 52
    [13, 5, 7], // 53
    [15, 4, 9], // 54
    [14, 3, 10], // 55
    [16, 2, 10], // 56
    [17, 1, 10], // 57
    [18, 7, 2], // 58
    [19, 9, 1], // 59
    [20, 4, 5], // 60
    [21, 3, 3], // 61
    [22, 5, 5], // 62
    [23, 2, 5], // 63
    [24, 1, 10], // 64
    [25, 9, 4], // 65
    [26, 8, 8], // 66
    [27, 7, 5], // 67
    [28, 6, 3], // 68
    [29, 5, 8], // 69
    [30, 4, 7], // 70
    [31, 5, 6], // 71
    [32, 6, 9], // 72
    [33, 4, 6], // 73
    [34, 8, 10], // 74
    [35, 7, 2], // 75
    [36, 6, 6], // 76
    [37, 5, 5], // 77
    [38, 9, 4], // 78
    [39, 8, 7], // 79
    [0, 7, 4], // 80
    [1, 6, 3], // 81
    [2, 5, 5], // 82
    [3, 8, 8], // 83
    [4, 7, 6], // 84
    [5, 9, 7], // 85
    [6, 4, 9], // 86
    [7, 3, 10], // 87
    [8, 5, 10], // 88
    [9, 2, 10], // 89
    [0, 9, 7], // 90
    [0, 8, 4], // 91
    [12, 8, 5], // 92
    [13, 7, 3], // 93
    [15, 6, 5], // 94
    [14, 5, 5], // 95
    [16, 4, 10], // 96
    [17, 5, 4], // 97
    [18, 6, 8], // 98
    [19, 4, 5], // 99
    [0, 8, 3], // 100
    [37, 7, 8], // 101
    [11, 6, 9], // 102
    [23, 5, 6], // 103
    [24, 9, 9], // 104
    [25, 8, 6], // 105
    [26, 7, 10], // 106
    [27, 6, 2], // 107
    [0, 8, 6], // 108
    [29, 8, 5], // 109
    [30, 7, 4], // 110
    [31, 9, 7], // 111
    [32, 4, 4], // 112
    [33, 3, 3], // 113
    [34, 5, 5], // 114
    [35, 2, 8], // 115
    [36, 1, 6], // 116
    [37, 9, 7], // 117
    [38, 8, 9], // 118
    [0, 7, 10], // 119
    [11, 8, 5], // 120
    [0, 5, 9], // 121
    [0, 4, 9], // 122
    [0, 8, 1], // 123
    [0, 6, 5], // 124
    [0, 9, 3], // 125
    [0, 8, 5], // 126
    [0, 7, 5], // 127
    [8, 6, 10], // 128
    [9, 5, 4], // 129
    [10, 9, 8], // 130
    [11, 8, 5], // 131
    [12, 7, 3], // 132
    [0, 6, 8], // 133
    [0, 7, 7], // 134
    [14, 8, 6], // 135
    [16, 7, 9], // 136
    [0, 9, 6], // 137
    [18, 4, 10], // 138
    [19, 3, 2], // 139
    [0, 5, 6], // 140
    [0, 4, 5], // 141
    [0, 6, 4], // 142
    [0, 9, 7], // 143
    [24, 8, 4], // 144
    [25, 7, 3], // 145
    [26, 6, 5], // 146
    [27, 5, 8], // 147
    [28, 4, 6], // 148
    [29, 5, 7], // 149
    [30, 6, 9], // 150
    [31, 4, 10], // 151
    [32, 8, 10], // 152
    [33, 7, 10], // 153
    [34, 6, 2], // 154
    [35, 5, 1], // 155
    [36, 9, 5], // 156
    [0, 8, 3], // 157
    [0, 7, 5], // 158
    [0, 6, 5], // 159
    [0, 5, 10], // 160
    [37, 8, 4], // 161
    [0, 7, 8], // 162
    [3, 9, 5], // 163
    [4, 4, 3], // 164
    [5, 3, 8], // 165
    [37, 5, 7], // 166
    [0, 2, 6], // 167
    [0, 1, 9], // 168
    [13, 9, 6], // 169
    [10, 8, 10], // 170
    [11, 7, 2], // 171
    [12, 6, 6], // 172
    [13, 5, 5], // 173
    [15, 4, 4], // 174
    [14, 5, 7], // 175
    [16, 6, 4], // 176
    [17, 4, 3], // 177
    [18, 8, 5], // 178
    [19, 7, 8], // 179
];

impl DriverTable {
    pub const fn new() -> Self {
        let mut drivers = [const { Driver::new(0, [0, 0, 0]) }; 180];
        let mut names = [const { DriverName::new(0, [0, 0, 0]) }; 180];
        let mut k = 0;
        while k < 180 {
            drivers[k] = Driver::new(k as u32, DRIVER_ROWS[k]);
            names[k] = DriverName::new(k as u32, NAME_ROWS[k]);
            k += 1;
        }
        DriverTable {
            drivers,
            end: PlainDword::new(0xFFFF_FFFF),
            names,
        }
    }
}

/// The string id of name entry `entry` (a driver's first dword). The original does not
/// check the entry and reads on past the 180 (fixed: refused).
pub fn name_string(menus: &Menus, entry: u32) -> Result<u32, crate::errors::BadInput> {
    Ok(menus
        .cars
        .drivers
        .names
        .get(entry as usize)
        .ok_or(crate::errors::BadInput(
            "a driver's name entry past the 180",
        ))?
        .string
        .get())
}

/// IDA 0x73B674, the first byte of the piece.
#[cfg(test)]
pub const START: usize = 0x73B674;
/// IDA 0x742C98, the end of the piece.
#[cfg(test)]
pub const END: usize = 0x742C98;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drivers_start_as_the_games() {
        assert_eq!(size_of::<DriverTable>(), END - START);
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::menu::owner::menus().cars.drivers,
        );
    }
}
