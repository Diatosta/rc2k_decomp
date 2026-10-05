use std::mem::{offset_of, size_of};

/// One 0xA4-byte row of the record table. `packed`: the table base is not 4-byte aligned.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Record {
    pub tag: u32, // 0x00: 4 = section header
    pub field_04: u32,
    pub field_08: u32,
    pub field_0c: u32, // optional measurement, 0 = absent
    pub field_10: i32,
    pub field_14: u32,
    pub field_18: u32,
    pub temperature_1c: i32,
    pub field_20: i32,
    pub field_24: u32,
    pub field_28: u32,
    pub field_2c: u32,
    pub field_30: u32,
    pub field_34: u32,
    pub field_38: u32,
    pub field_3c: u32,
    pub rest_40: [u8; 0x64],
}

const _: () = assert!(size_of::<Record>() == 0xA4);
const _: () = assert!(offset_of!(Record, temperature_1c) == 0x1C);
const _: () = assert!(offset_of!(Record, field_3c) == 0x3C);
