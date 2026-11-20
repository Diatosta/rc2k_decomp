#![no_main]

// Race messages stored one after another into a player's inbox, then handed over: the inbox's
// records stay within their room whatever the sequence bytes and lengths are.
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    use rc2k_core::net::inbox::{Inbox, RECORDS_LEN, hand_over, store};
    let mut inbox = Inbox {
        count: 0,
        records: vec![0; RECORDS_LEN],
        oldest: 0,
        newest: 0,
        time: 0,
    };
    let mut rest = data;
    while rest.len() >= 6 {
        let len = usize::from(rest[5]).min(rest.len() - 6);
        let time = u32::from_le_bytes([rest[1], rest[2], rest[3], rest[4]]);
        let _ = store(&mut inbox, rest[0], time, &rest[6..6 + len]);
        rest = &rest[6 + len..];
    }
    let _ = hand_over(&mut inbox);
});
