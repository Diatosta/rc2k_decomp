#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if data.len() < 6 { return; }
    let mut inbox = vec![0u8; 2 * rc2k_core::net::inbox::COPIED];
    let mut rest = data;
    while rest.len() >= 6 {
        let len = usize::from(rest[5]).min(rest.len() - 6);
        let time = u32::from_le_bytes([rest[1], rest[2], rest[3], rest[4]]);
        let _ = rc2k_core::net::inbox::store(&mut inbox, rest[0], time, &rest[6..6 + len]);
        rest = &rest[6 + len..];
    }
    let _ = rc2k_core::net::inbox::hand_over(&mut inbox);
});
