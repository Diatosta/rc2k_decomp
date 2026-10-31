#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if data.len() < 4 { return; }
    let id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) & 0xFFFF;
    let table = rc2k_core::strings::table::StringTable::new(&data[4..]);
    let _ = table.entry_count();
    let mut out = Vec::new();
    let _ = table.expand(id, &mut out);
});
