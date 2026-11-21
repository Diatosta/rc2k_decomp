#![no_main]

// A film's packets one after another through one decoder: each is two bytes of length,
// then the packet, its start code put right so that the fuzzer gets past it.
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let mut decoder = rc2k_core::film::indeo4::Decoder::new();
    let mut rest = data;
    let mut sum = 0u64;
    while let [low, high, tail @ ..] = rest {
        let length = usize::from(u16::from_le_bytes([*low, *high])).min(tail.len());
        let (packet, tail) = tail.split_at(length);
        rest = tail;
        let mut packet = packet.to_vec();
        if let [first, second, third, ..] = packet.as_mut_slice() {
            *first = 0xF8;
            *second = 0xFF;
            *third |= 3;
        }
        if let Ok(Some(picture)) = decoder.decode(&packet) {
            sum += picture.planes.iter().flatten().map(|&b| u64::from(b)).sum::<u64>();
        }
    }
    std::hint::black_box(sum);
});
