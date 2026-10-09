//! The game's unit system and the string ids of its unit labels.

use crate::framework::{game_ptr, game_read, rva};

// 0 = imperial (Fahrenheit, miles) and non-zero = metric.
const UNITS_RVA: usize = rva(0x51DE2C);
const TEMPERATURE_LABEL_IDS_RVA: usize = rva(0x51EB84); // g_temperatureLabelIds[units]
const DISTANCE_LABEL_IDS_RVA: usize = rva(0x51EB88); // g_distanceLabelIds[units]

/// 0 = imperial, non-zero = metric. The game stores Celsius and miles.
pub fn units() -> u32 {
    unsafe { game_read(UNITS_RVA) }
}

pub fn is_metric() -> bool {
    units() != 0
}

fn label_id(table_rva: usize, units: u32) -> u32 {
    unsafe {
        game_ptr::<u16>(table_rva)
            .wrapping_add(units as usize)
            .read_volatile() as u32
    }
}
pub fn temperature_label_id(units: u32) -> u32 {
    label_id(TEMPERATURE_LABEL_IDS_RVA, units)
}
pub fn distance_label_id(units: u32) -> u32 {
    label_id(DISTANCE_LABEL_IDS_RVA, units)
}
