#![no_main]

// An MPEG audio stream cut into frames and decoded, and the whole of it as one packet.
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    use rc2k_core::film::mp2::{AudioDecoder, cut};
    let mut decoder = AudioDecoder::new();
    let mut pcm = Vec::new();
    for packet in cut(data) {
        let _ = decoder.decode(&data[packet], &mut pcm);
    }
    let _ = decoder.decode(data, &mut pcm);
    let _ = rc2k_core::film::mpeg_film::demux(data);
    std::hint::black_box(pcm);
});
